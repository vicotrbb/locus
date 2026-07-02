//! Validation helpers that combine Locus probe verdicts.

#[cfg(target_os = "linux")]
pub mod linux {
    //! Linux placement validation gate helpers.

    use std::fmt;

    use locus_observe::{
        parse_numa_placement_proof_output, parse_numa_placement_readiness_output,
        NumaPlacementProof, NumaPlacementProofOutputParseError, NumaPlacementProofStatus,
        NumaPlacementReadinessOutputParseError, NumaPlacementValidationReadiness,
    };
    use locus_sys::linux::{
        parse_linux_numa_policy_readiness_output, LinuxNumaPolicyReadiness,
        LinuxNumaPolicyReadinessOutputParseError,
    };

    /// Probe outputs required for a combined placement validation gate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PlacementValidationOutputs<'a> {
        /// Full output from `locus-sys --example mbind_region`.
        pub memory_policy_output: &'a str,
        /// Full output from `locus-observe --example locality_environment`.
        pub placement_readiness_output: &'a str,
        /// Full output from `locus-alloc --example mapped_scratch_bind`.
        pub placement_proof_output: &'a str,
    }

    /// Combined placement validation gate status.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PlacementValidationGateStatus {
        /// All required proof conditions are satisfied.
        Verified,
        /// The environment is not ready to prove placement.
        NotReady,
        /// Evidence was available but did not prove placement.
        Unverified,
        /// Primary placement proof evidence was unavailable.
        Unavailable,
    }

    impl PlacementValidationGateStatus {
        /// Returns a stable machine-readable status string.
        #[must_use]
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Verified => "verified",
                Self::NotReady => "not_ready",
                Self::Unverified => "unverified",
                Self::Unavailable => "unavailable",
            }
        }
    }

    impl fmt::Display for PlacementValidationGateStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    /// Reason for the combined placement validation gate status.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PlacementValidationGateReason {
        /// All required proof conditions are satisfied.
        Verified,
        /// Linux memory policy application readiness was not ready.
        MemoryPolicyNotReady,
        /// Locality evidence readiness was not ready.
        PlacementEvidenceNotReady,
        /// Placement proof evidence was unavailable.
        PlacementProofUnavailable,
        /// Placement proof evidence was present but unverified.
        PlacementProofUnverified,
    }

    impl PlacementValidationGateReason {
        /// Returns a stable machine-readable reason string.
        #[must_use]
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Verified => "verified",
                Self::MemoryPolicyNotReady => "memory_policy_not_ready",
                Self::PlacementEvidenceNotReady => "placement_evidence_not_ready",
                Self::PlacementProofUnavailable => "placement_proof_unavailable",
                Self::PlacementProofUnverified => "placement_proof_unverified",
            }
        }
    }

    impl fmt::Display for PlacementValidationGateReason {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    /// Parsed combined placement validation gate verdict.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PlacementValidationGate {
        /// Final gate status.
        pub status: PlacementValidationGateStatus,
        /// Reason for the status.
        pub reason: PlacementValidationGateReason,
        /// Parsed Linux memory-policy readiness.
        pub memory_policy: LinuxNumaPolicyReadiness,
        /// Parsed locality evidence readiness.
        pub placement_readiness: NumaPlacementValidationReadiness,
        /// Parsed mapped arena placement proof.
        pub placement_proof: NumaPlacementProof,
    }

    impl PlacementValidationGate {
        /// Builds a combined gate from parsed verdicts.
        #[must_use]
        pub fn from_verdicts(
            memory_policy: LinuxNumaPolicyReadiness,
            placement_readiness: NumaPlacementValidationReadiness,
            placement_proof: NumaPlacementProof,
        ) -> Self {
            let (status, reason) = if !memory_policy.is_ready() {
                (
                    PlacementValidationGateStatus::NotReady,
                    PlacementValidationGateReason::MemoryPolicyNotReady,
                )
            } else if !placement_readiness.is_ready() {
                (
                    PlacementValidationGateStatus::NotReady,
                    PlacementValidationGateReason::PlacementEvidenceNotReady,
                )
            } else {
                match placement_proof.status {
                    NumaPlacementProofStatus::Verified => (
                        PlacementValidationGateStatus::Verified,
                        PlacementValidationGateReason::Verified,
                    ),
                    NumaPlacementProofStatus::Unverified => (
                        PlacementValidationGateStatus::Unverified,
                        PlacementValidationGateReason::PlacementProofUnverified,
                    ),
                    NumaPlacementProofStatus::Unavailable => (
                        PlacementValidationGateStatus::Unavailable,
                        PlacementValidationGateReason::PlacementProofUnavailable,
                    ),
                }
            };

            Self {
                status,
                reason,
                memory_policy,
                placement_readiness,
                placement_proof,
            }
        }

        /// Returns true only when all proof conditions are satisfied.
        #[must_use]
        pub fn is_verified(self) -> bool {
            self.status == PlacementValidationGateStatus::Verified
        }
    }

    /// Errors from parsing combined placement validation probe output.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PlacementValidationGateParseError {
        /// Memory-policy readiness output was missing or malformed.
        MemoryPolicy(LinuxNumaPolicyReadinessOutputParseError),
        /// Placement evidence readiness output was missing or malformed.
        PlacementReadiness(NumaPlacementReadinessOutputParseError),
        /// Placement proof output was missing or malformed.
        PlacementProof(NumaPlacementProofOutputParseError),
    }

    impl fmt::Display for PlacementValidationGateParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::MemoryPolicy(source) => write!(f, "invalid memory policy output: {source}"),
                Self::PlacementReadiness(source) => {
                    write!(f, "invalid placement readiness output: {source}")
                }
                Self::PlacementProof(source) => {
                    write!(f, "invalid placement proof output: {source}")
                }
            }
        }
    }

    impl std::error::Error for PlacementValidationGateParseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::MemoryPolicy(source) => Some(source),
                Self::PlacementReadiness(source) => Some(source),
                Self::PlacementProof(source) => Some(source),
            }
        }
    }

    /// Parses probe outputs and returns the combined placement validation gate.
    ///
    /// # Errors
    ///
    /// Returns an error when any probe output is missing its final verdict line
    /// or contains a malformed final verdict line.
    pub fn evaluate_placement_validation_outputs(
        outputs: PlacementValidationOutputs<'_>,
    ) -> Result<PlacementValidationGate, PlacementValidationGateParseError> {
        let memory_policy = parse_linux_numa_policy_readiness_output(outputs.memory_policy_output)
            .map_err(PlacementValidationGateParseError::MemoryPolicy)?;
        let placement_readiness =
            parse_numa_placement_readiness_output(outputs.placement_readiness_output)
                .map_err(PlacementValidationGateParseError::PlacementReadiness)?;
        let placement_proof = parse_numa_placement_proof_output(outputs.placement_proof_output)
            .map_err(PlacementValidationGateParseError::PlacementProof)?;

        Ok(PlacementValidationGate::from_verdicts(
            memory_policy,
            placement_readiness,
            placement_proof,
        ))
    }

    #[cfg(test)]
    mod tests {
        use locus_observe::{
            NumaPlacementProof, NumaPlacementProofReason, NumaPlacementProofStatus,
            NumaPlacementValidationReadiness, NumaPlacementValidationReadinessReason,
            NumaPlacementValidationReadinessStatus,
        };
        use locus_sys::linux::{
            LinuxNumaPolicyReadiness, LinuxNumaPolicyReadinessReason,
            LinuxNumaPolicyReadinessStatus,
        };

        use super::{
            evaluate_placement_validation_outputs, PlacementValidationGate,
            PlacementValidationGateParseError, PlacementValidationGateReason,
            PlacementValidationGateStatus, PlacementValidationOutputs,
        };

        #[test]
        fn reports_verified_gate_from_ready_inputs() {
            let gate = PlacementValidationGate::from_verdicts(
                LinuxNumaPolicyReadiness {
                    status: LinuxNumaPolicyReadinessStatus::Ready,
                    reason: LinuxNumaPolicyReadinessReason::Ready,
                },
                NumaPlacementValidationReadiness {
                    status: NumaPlacementValidationReadinessStatus::Ready,
                    reason: NumaPlacementValidationReadinessReason::Ready,
                },
                NumaPlacementProof {
                    status: NumaPlacementProofStatus::Verified,
                    reason: NumaPlacementProofReason::Verified,
                },
            );

            assert_eq!(gate.status, PlacementValidationGateStatus::Verified);
            assert_eq!(gate.reason, PlacementValidationGateReason::Verified);
            assert_eq!(gate.status.to_string(), "verified");
            assert_eq!(gate.reason.to_string(), "verified");
            assert!(gate.is_verified());
        }

        #[test]
        fn prioritizes_readiness_before_proof_status() {
            let not_ready_policy = LinuxNumaPolicyReadiness {
                status: LinuxNumaPolicyReadinessStatus::NotReady,
                reason: LinuxNumaPolicyReadinessReason::PermissionDenied,
            };
            let ready_evidence = NumaPlacementValidationReadiness {
                status: NumaPlacementValidationReadinessStatus::Ready,
                reason: NumaPlacementValidationReadinessReason::Ready,
            };
            let unavailable_proof = NumaPlacementProof {
                status: NumaPlacementProofStatus::Unavailable,
                reason: NumaPlacementProofReason::NumaMapsUnavailable,
            };

            let gate = PlacementValidationGate::from_verdicts(
                not_ready_policy,
                ready_evidence,
                unavailable_proof,
            );

            assert_eq!(gate.status, PlacementValidationGateStatus::NotReady);
            assert_eq!(
                gate.reason,
                PlacementValidationGateReason::MemoryPolicyNotReady
            );
            assert!(!gate.is_verified());
        }

        #[test]
        fn evaluates_docker_probe_outputs_as_not_ready() {
            let gate = evaluate_placement_validation_outputs(PlacementValidationOutputs {
                memory_policy_output: "\
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=4
",
                placement_readiness_output: "\
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
",
                placement_proof_output: "\
mapping_start=0xffff98744000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
placement_proof=unavailable reason=numa_maps_unavailable
",
            })
            .expect("gate");

            assert_eq!(gate.status, PlacementValidationGateStatus::NotReady);
            assert_eq!(
                gate.reason,
                PlacementValidationGateReason::MemoryPolicyNotReady
            );
            assert_eq!(
                gate.placement_proof.status,
                NumaPlacementProofStatus::Unavailable
            );
        }

        #[test]
        fn reports_probe_output_parse_errors() {
            let error = evaluate_placement_validation_outputs(PlacementValidationOutputs {
                memory_policy_output: "mbind=ok\n",
                placement_readiness_output: "placement_validation_readiness=ready reason=ready\n",
                placement_proof_output: "placement_proof=verified reason=verified\n",
            })
            .expect_err("missing memory policy readiness");

            assert!(matches!(
                error,
                PlacementValidationGateParseError::MemoryPolicy(_)
            ));
        }
    }
}
