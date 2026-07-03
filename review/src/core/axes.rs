pub const AXES: [&str; 48] = [
    "urgency",
    "valence",
    "arousal",
    "agency",
    "self_relevance",
    "user_relevance",
    "goal_relevance",
    "temporal_near",
    "temporal_past",
    "concreteness",
    "social",
    "risk",
    "effort",
    "value",
    "novelty",
    "stability",
    "truth_assert",
    "desire",
    "obligation",
    "completion",
    "familiarity",
    "emotional_charge",
    "scope",
    "priority",
    "architecture_relevance",
    "memory_relevance",
    "language_relevance",
    "implementation_relevance",
    "contradiction_relevance",
    "uncertainty_relevance",
    "power_relevance",
    "autonomy",
    "tool_relevance",
    "reasoning_relevance",
    "planning_relevance",
    "identity_relevance",
    "preference_relevance",
    "constraint_relevance",
    "project_relevance",
    "learning_relevance",
    "recency",
    "persistence",
    "confidence_proxy",
    "activation_bias",
    "attention",
    "specificity",
    "context_relevance",
    "safety_relevance",
];

pub fn axis_index(name: &str) -> Option<usize> {
    AXES.iter().position(|axis| *axis == name)
}

pub fn axis_name(idx: usize) -> Option<&'static str> {
    AXES.get(idx).copied()
}

pub fn default_axes() -> Vec<String> {
    AXES.iter().map(|axis| (*axis).to_string()).collect()
}
