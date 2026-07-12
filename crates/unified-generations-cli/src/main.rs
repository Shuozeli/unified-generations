use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use unified_generations::{
    AgentPlanClient, AgentPlanConfig, ArkCliConfig, AudioFormat, DEFAULT_ANTHROPIC_VERSION,
    DEFAULT_PLAN_BASE_URL, DEFAULT_TTS_RESOURCE_ID, DEFAULT_TTS_URL, ImageGenerationRequest,
    ImageOutputFormat, ImageSize, LlmMessageRequest, TtsRequest, arkcli_config_path,
    load_arkcli_config, masked_api_key, tts_voice_presets, write_arkcli_config,
};

#[derive(Debug, Parser)]
#[command(version, about = "Volcengine Doubao Ark Agent Plan CLI")]
struct Args {
    #[arg(long, env = "DOUBAO_ARK_AGENT_PLAN_API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {
        #[arg(long, env = "DOUBAO_ARK_AGENT_PLAN_API_KEY")]
        api_key: Option<String>,
        #[arg(long, default_value = DEFAULT_PLAN_BASE_URL)]
        plan_base_url: String,
        #[arg(long, default_value = DEFAULT_TTS_URL)]
        tts_url: String,
        #[arg(long, default_value = DEFAULT_TTS_RESOURCE_ID)]
        tts_resource_id: String,
        #[arg(long, default_value = DEFAULT_ANTHROPIC_VERSION)]
        anthropic_version: String,
        #[arg(long)]
        force: bool,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Chat {
        prompt: String,
        #[arg(long, default_value = "doubao-seed-2.0-mini")]
        model: String,
        #[arg(long, default_value_t = 1024)]
        max_tokens: u32,
    },
    Image {
        prompt: String,
        #[arg(long, default_value = "doubao-seedream-5.0-lite")]
        model: String,
        #[arg(long, value_enum, default_value_t = CliImageSize::TwoK)]
        size: CliImageSize,
    },
    Tts {
        text: String,
        #[arg(long, default_value = "zh_female_gaolengyujie_uranus_bigtts")]
        speaker: String,
        #[arg(long, default_value = "speech.mp3")]
        out: PathBuf,
    },
    Voices {
        #[arg(long)]
        gender: Option<String>,
        #[arg(long)]
        category: Option<String>,
    },
    TtsProbe {
        #[arg(long, default_value = "你好，欢迎使用语音合成服务。")]
        text: String,
        #[arg(long, default_value = "out/tts-probe")]
        out_dir: PathBuf,
        #[arg(long, default_value_t = 6)]
        limit: usize,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Path,
    Show,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliImageSize {
    OneK,
    TwoK,
    FourK,
}

impl From<CliImageSize> for ImageSize {
    fn from(value: CliImageSize) -> Self {
        match value {
            CliImageSize::OneK => ImageSize::OneK,
            CliImageSize::TwoK => ImageSize::TwoK,
            CliImageSize::FourK => ImageSize::FourK,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init {
            api_key,
            plan_base_url,
            tts_url,
            tts_resource_id,
            anthropic_version,
            force,
        } => {
            let path = arkcli_config_path()?;
            if path.exists() && !force {
                anyhow::bail!(
                    "{} already exists; pass --force to overwrite it",
                    path.display()
                );
            }

            let api_key = resolve_init_api_key(api_key.or(args.api_key))?;
            let mut config = ArkCliConfig::new(api_key);
            config.plan_base_url = plan_base_url;
            config.tts_url = tts_url;
            config.tts_resource_id = tts_resource_id;
            config.anthropic_version = anthropic_version;

            let path = write_arkcli_config(&config)?;
            println!("wrote {}", path.display());
            println!("api_key {}", masked_api_key(&config.api_key));
            println!("plan_base_url {}", config.plan_base_url);
            println!("tts_url {}", config.tts_url);
            println!("tts_resource_id {}", config.tts_resource_id);
            println!("anthropic_version {}", config.anthropic_version);
        }
        Command::Config { command } => match command {
            ConfigCommand::Path => {
                println!("{}", arkcli_config_path()?.display());
            }
            ConfigCommand::Show => {
                let path = arkcli_config_path()?;
                let config = load_arkcli_config()?.with_context(|| {
                    format!("{} does not exist; run init first", path.display())
                })?;
                println!("path {}", path.display());
                println!("api_key {}", masked_api_key(&config.api_key));
                println!("plan_base_url {}", config.plan_base_url);
                println!("tts_url {}", config.tts_url);
                println!("tts_resource_id {}", config.tts_resource_id);
                println!("anthropic_version {}", config.anthropic_version);
            }
        },
        Command::Chat {
            prompt,
            model,
            max_tokens,
        } => {
            let client = make_client(args.api_key)?;
            let mut request = LlmMessageRequest::new(model, prompt);
            request.max_tokens = max_tokens;
            let response = client.send_message(&request).await?;
            println!("{}", response.text());
        }
        Command::Image {
            prompt,
            model,
            size,
        } => {
            let client = make_client(args.api_key)?;
            let mut request = ImageGenerationRequest::new(prompt);
            request.model = model;
            request.size = size.into();
            request.output_format = ImageOutputFormat::Png;
            let response = client.generate_image(&request).await?;
            println!("{}", serde_json::to_string_pretty(&response.data)?);
        }
        Command::Tts { text, speaker, out } => {
            let client = make_client(args.api_key)?;
            let mut request = TtsRequest::new(text, speaker);
            request.req_params.audio_params.format = AudioFormat::Mp3;
            let response = client.synthesize_speech(&request).await?;
            std::fs::write(&out, response.audio)
                .with_context(|| format!("failed to write {}", out.display()))?;
            println!("{}", out.display());
        }
        Command::Voices { gender, category } => {
            for voice in tts_voice_presets()
                .iter()
                .filter(|voice| {
                    gender
                        .as_deref()
                        .is_none_or(|gender| voice.gender == gender)
                })
                .filter(|voice| {
                    category
                        .as_deref()
                        .is_none_or(|category| voice.category == category)
                })
            {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    voice.id, voice.display_name, voice.locale, voice.gender, voice.category
                );
            }
        }
        Command::TtsProbe {
            text,
            out_dir,
            limit,
        } => {
            let client = make_client(args.api_key)?;
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("failed to create {}", out_dir.display()))?;
            for voice in tts_voice_presets().iter().take(limit) {
                let mut request = TtsRequest::new(text.clone(), voice.id);
                request.req_params.audio_params.format = AudioFormat::Mp3;
                let out = out_dir.join(format!("{}.mp3", sanitize_filename(voice.id)));
                match client.synthesize_speech(&request).await {
                    Ok(response) => {
                        std::fs::write(&out, response.audio)
                            .with_context(|| format!("failed to write {}", out.display()))?;
                        println!("ok\t{}\t{}", voice.id, out.display());
                    }
                    Err(error) => {
                        println!("err\t{}\t{}", voice.id, error);
                    }
                }
            }
        }
    }

    Ok(())
}

fn make_client(api_key: Option<String>) -> anyhow::Result<AgentPlanClient> {
    let config = AgentPlanConfig::from_sources(api_key)?;
    Ok(AgentPlanClient::new(config)?)
}

fn resolve_init_api_key(api_key: Option<String>) -> anyhow::Result<String> {
    if let Some(api_key) = api_key {
        return Ok(api_key);
    }
    if let Ok(api_key) = std::env::var("ARK_AGENT_PLAN_API_KEY") {
        return Ok(api_key);
    }
    if !std::io::stdin().is_terminal() {
        anyhow::bail!("API key is required; pass --api-key or set DOUBAO_ARK_AGENT_PLAN_API_KEY");
    }

    print!("Ark Agent Plan API key: ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let api_key = input.trim().to_string();
    if api_key.is_empty() {
        anyhow::bail!("API key is required");
    }
    Ok(api_key)
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
