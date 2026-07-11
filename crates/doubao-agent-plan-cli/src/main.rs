use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use doubao_agent_plan::{
    AgentPlanClient, AgentPlanConfig, AudioFormat, ImageGenerationRequest, ImageOutputFormat,
    ImageSize, LlmMessageRequest, TtsRequest, tts_voice_presets,
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
    let config = match api_key {
        Some(api_key) => AgentPlanConfig::new(api_key),
        None => AgentPlanConfig::from_env()?,
    };
    Ok(AgentPlanClient::new(config)?)
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
