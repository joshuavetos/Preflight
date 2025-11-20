use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub ports: Vec<u16>,
    pub docker_compose: bool,
    pub docker_run: bool,
    pub docker_build: bool,
    pub raw: String,
}

pub fn parse_command(raw: &str) -> ParsedCommand {
    let normalized = raw.to_lowercase();
    let port_regex =
        Regex::new(r"(?P<port>\d{2,5})").expect("regex construction invariant: constant pattern");

    let mut ports = Vec::new();
    for cap in port_regex.captures_iter(&normalized) {
        if let Some(port_match) = cap.name("port") {
            if let Ok(port) = port_match.as_str().parse::<u16>() {
                ports.push(port);
            }
        }
    }

    let docker_compose =
        normalized.contains("docker compose") || normalized.contains("docker-compose");
    let docker_run = normalized.contains("docker run");
    let docker_build = normalized.contains("docker build");

    ParsedCommand {
        ports,
        docker_compose,
        docker_run,
        docker_build,
        raw: raw.to_string(),
    }
}
