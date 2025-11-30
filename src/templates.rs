pub struct ConfigTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    template: &'static str,
    pipeline_overrides: &'static [(&'static str, &'static str)],
    settings: TemplateSettings,
}

#[derive(Clone, Copy)]
pub struct TemplateSettings {
    pub langfuse_enable: bool,
    pub logging_level: &'static str,
    pub development: bool,
}

const ENGINE_SECTION: &str = include_str!("../config_templates/common/engine.yaml");
const PIPELINE_TEMPLATE: &str = include_str!("../config_templates/common/pipeline.yaml");
const SETTINGS_TEMPLATE: &str = include_str!("../config_templates/common/settings.yaml");

const PIPELINE_DEFAULT_BINDINGS: &[(&str, &str)] = &[
    ("{{LLM_SQL_ANSWER}}", "litellm_llm.default"),
    ("{{LLM_DATA_ASSISTANCE}}", "litellm_llm.default"),
    ("{{LLM_CHART_GENERATION}}", "litellm_llm.default"),
    ("{{LLM_CHART_ADJUSTMENT}}", "litellm_llm.default"),
    ("{{LLM_SQL_GENERATION_REASONING}}", "litellm_llm.default"),
    (
        "{{LLM_FOLLOWUP_SQL_GENERATION_REASONING}}",
        "litellm_llm.default",
    ),
];

const NO_OVERRIDES: &[(&str, &str)] = &[];
const OVERRIDES_DEEPSEEK: &[(&str, &str)] = &[
    ("{{LLM_SQL_ANSWER}}", "litellm_llm.deepseek/deepseek-chat"),
    (
        "{{LLM_DATA_ASSISTANCE}}",
        "litellm_llm.deepseek/deepseek-chat",
    ),
    (
        "{{LLM_SQL_GENERATION_REASONING}}",
        "litellm_llm.deepseek/deepseek-reasoner",
    ),
    (
        "{{LLM_FOLLOWUP_SQL_GENERATION_REASONING}}",
        "litellm_llm.deepseek/deepseek-reasoner",
    ),
];
const OVERRIDES_GEMINI_CHART: &[(&str, &str)] = &[
    (
        "{{LLM_CHART_GENERATION}}",
        "litellm_llm.gemini-llm-for-chart",
    ),
    (
        "{{LLM_CHART_ADJUSTMENT}}",
        "litellm_llm.gemini-llm-for-chart",
    ),
];
const OVERRIDES_QWEN3: &[(&str, &str)] = &[
    ("{{LLM_SQL_ANSWER}}", "litellm_llm.qwen3-fast"),
    ("{{LLM_DATA_ASSISTANCE}}", "litellm_llm.qwen3-fast"),
    (
        "{{LLM_SQL_GENERATION_REASONING}}",
        "litellm_llm.qwen3-thinking",
    ),
    (
        "{{LLM_FOLLOWUP_SQL_GENERATION_REASONING}}",
        "litellm_llm.qwen3-thinking",
    ),
];
const OVERRIDES_ZHIPU: &[(&str, &str)] = &[
    ("{{LLM_SQL_ANSWER}}", "litellm_llm.glm45-fast"),
    ("{{LLM_DATA_ASSISTANCE}}", "litellm_llm.glm45-fast"),
    (
        "{{LLM_SQL_GENERATION_REASONING}}",
        "litellm_llm.glm45-thinking",
    ),
    (
        "{{LLM_FOLLOWUP_SQL_GENERATION_REASONING}}",
        "litellm_llm.glm45-thinking",
    ),
];

const SETTINGS_OPENAI: TemplateSettings = TemplateSettings {
    langfuse_enable: false,
    logging_level: "INFO",
    development: false,
};
const SETTINGS_DEBUG_FALSE: TemplateSettings = TemplateSettings {
    langfuse_enable: true,
    logging_level: "DEBUG",
    development: false,
};
const SETTINGS_DEBUG_TRUE: TemplateSettings = TemplateSettings {
    langfuse_enable: true,
    logging_level: "DEBUG",
    development: true,
};
const SETTINGS_OPEN_ROUTER: TemplateSettings = TemplateSettings {
    langfuse_enable: true,
    logging_level: "DEBUG",
    development: false,
};

const CONFIG_OPENAI: ConfigTemplate = ConfigTemplate {
    key: "openai",
    name: "OpenAI (GPT-4o mini)",
    description: "Use OpenAI gpt-4o-mini with text-embedding-3-large",
    template: include_str!("../config_templates/providers/openai.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_OPENAI,
};
const CONFIG_ANTHROPIC: ConfigTemplate = ConfigTemplate {
    key: "anthropic",
    name: "Anthropic Claude 3.7 Sonnet",
    description: "Anthropic Claude via api.anthropic.com",
    template: include_str!("../config_templates/providers/anthropic.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_FALSE,
};
const CONFIG_AZURE: ConfigTemplate = ConfigTemplate {
    key: "azure",
    name: "Azure OpenAI",
    description: "Azure OpenAI deployment using gpt-4",
    template: include_str!("../config_templates/providers/azure.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_FALSE,
};
const CONFIG_BEDROCK: ConfigTemplate = ConfigTemplate {
    key: "bedrock",
    name: "AWS Bedrock",
    description: "Amazon Bedrock Claude Sonnet + Titan embeddings",
    template: include_str!("../config_templates/providers/bedrock.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_FALSE,
};
const CONFIG_DEEPSEEK: ConfigTemplate = ConfigTemplate {
    key: "deepseek",
    name: "DeepSeek",
    description: "DeepSeek reasoning and chat models via api.deepseek.com",
    template: include_str!("../config_templates/providers/deepseek.yaml"),
    pipeline_overrides: OVERRIDES_DEEPSEEK,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_GOOGLE_AI: ConfigTemplate = ConfigTemplate {
    key: "google_ai_studio",
    name: "Google Gemini (AI Studio)",
    description: "Gemini 2.0 Flash via Google AI Studio",
    template: include_str!("../config_templates/providers/google_ai_studio.yaml"),
    pipeline_overrides: OVERRIDES_GEMINI_CHART,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_GOOGLE_VERTEX: ConfigTemplate = ConfigTemplate {
    key: "google_vertexai",
    name: "Google Gemini (Vertex AI)",
    description: "Gemini 2.5 Flash via Vertex AI",
    template: include_str!("../config_templates/providers/google_vertexai.yaml"),
    pipeline_overrides: OVERRIDES_GEMINI_CHART,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_GROK: ConfigTemplate = ConfigTemplate {
    key: "grok",
    name: "xAI Grok",
    description: "xAI Grok 3 via api.x.ai",
    template: include_str!("../config_templates/providers/grok.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_GROQ: ConfigTemplate = ConfigTemplate {
    key: "groq",
    name: "Groq Llama 3.3",
    description: "Groq API with Llama 3.3 70B specdec",
    template: include_str!("../config_templates/providers/groq.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_LM_STUDIO: ConfigTemplate = ConfigTemplate {
    key: "lm_studio",
    name: "LM Studio",
    description: "Local LM Studio endpoint (phi-4 + nomic embeddings)",
    template: include_str!("../config_templates/providers/lm_studio.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_OLLAMA: ConfigTemplate = ConfigTemplate {
    key: "ollama",
    name: "Ollama",
    description: "Local Ollama with phi4:14b",
    template: include_str!("../config_templates/providers/ollama.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_OPEN_ROUTER: ConfigTemplate = ConfigTemplate {
    key: "open_router",
    name: "OpenRouter",
    description: "OpenRouter Claude 3.7 Sonnet",
    template: include_str!("../config_templates/providers/open_router.yaml"),
    pipeline_overrides: NO_OVERRIDES,
    settings: SETTINGS_OPEN_ROUTER,
};
const CONFIG_QWEN3: ConfigTemplate = ConfigTemplate {
    key: "qwen3",
    name: "Qwen3",
    description: "Qwen3 via OpenRouter with thinking and fast modes",
    template: include_str!("../config_templates/providers/qwen3.yaml"),
    pipeline_overrides: OVERRIDES_QWEN3,
    settings: SETTINGS_DEBUG_TRUE,
};
const CONFIG_ZHIPU: ConfigTemplate = ConfigTemplate {
    key: "zhipu",
    name: "Zhipu GLM-4.5",
    description: "Zhipu AI GLM-4.5 with thinking/fast variants",
    template: include_str!("../config_templates/providers/zhipu.yaml"),
    pipeline_overrides: OVERRIDES_ZHIPU,
    settings: SETTINGS_DEBUG_TRUE,
};

pub const CONFIG_TEMPLATES: &[ConfigTemplate] = &[
    CONFIG_OPENAI,
    CONFIG_ANTHROPIC,
    CONFIG_AZURE,
    CONFIG_BEDROCK,
    CONFIG_DEEPSEEK,
    CONFIG_GOOGLE_AI,
    CONFIG_GOOGLE_VERTEX,
    CONFIG_GROK,
    CONFIG_GROQ,
    CONFIG_LM_STUDIO,
    CONFIG_OLLAMA,
    CONFIG_OPEN_ROUTER,
    CONFIG_QWEN3,
    CONFIG_ZHIPU,
];

impl ConfigTemplate {
    pub fn render(&self) -> String {
        let mut content = self.template.replace("{{ENGINE_SECTION}}", ENGINE_SECTION);

        let pipeline = render_pipeline(self.pipeline_overrides);
        content = content.replace("{{PIPELINE_SECTION}}", &pipeline);

        let settings = render_settings(&self.settings);
        content.replace("{{SETTINGS_SECTION}}", &settings)
    }
}

fn render_pipeline(overrides: &[(&'static str, &'static str)]) -> String {
    let mut rendered = PIPELINE_TEMPLATE.to_string();

    for (placeholder, default_value) in PIPELINE_DEFAULT_BINDINGS {
        let value = overrides
            .iter()
            .find(|(key, _)| key == placeholder)
            .map(|(_, value)| *value)
            .unwrap_or(*default_value);
        rendered = rendered.replace(placeholder, value);
    }

    rendered
}

fn render_settings(settings: &TemplateSettings) -> String {
    SETTINGS_TEMPLATE
        .replace(
            "{{LANGFUSE_ENABLE}}",
            bool_literal(settings.langfuse_enable),
        )
        .replace("{{LOGGING_LEVEL}}", settings.logging_level)
        .replace("{{DEVELOPMENT}}", bool_literal(settings.development))
}

fn bool_literal(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
