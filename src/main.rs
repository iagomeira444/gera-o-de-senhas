use axum::{
    Router,
    body::Bytes,
    extract::DefaultBodyLimit,
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use rand::{Rng, rngs::OsRng, seq::SliceRandom};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{HashMap, HashSet},
    env,
    net::SocketAddr,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::SystemTime,
    time::{Duration, Instant},
};

const MINUSCULAS: &str = "abcdefghijklmnopqrstuvwxyz";
const MAIUSCULAS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMEROS: &str = "0123456789";
const SIMBOLOS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOS: &str = "Il1O0";
const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const RATE_LIMIT_MAX_REQUESTS: u32 = 120;
const BODY_LIMIT_BYTES: usize = 8 * 1024;
const TRUSTED_PROXY_HOPS: usize = 1;
const DEFAULT_SITE_NAME: &str = "Gerador de Senhas";
const DEFAULT_SITE_DESCRIPTION: &str = "Gerador de senhas seguras online com criacao instantanea de senhas aleatorias, memoraveis e PINs fortes.";
const DEFAULT_PORT: u16 = 3000;

const PALAVRAS_CURTAS: &[&str] = &[
    "sol", "lua", "mar", "rio", "ceu", "mel", "flor", "vale", "lago", "onda", "raiz", "ramo",
    "mapa", "trilha", "ponte", "farol", "chave", "arco", "vela", "jade", "opala", "anil", "cobre",
    "bronze", "aco", "ritmo", "verso", "conto", "lenda", "saga", "runa", "sinal", "pulso",
    "portal", "vigia", "duna", "orca", "lince", "zenite", "nadir",
];

const CATEGORIA_NATUREZA: &[&str] = &[
    "bosque", "campo", "serra", "praia", "selva", "jardim", "planta", "fruta", "montanha",
    "floresta", "orvalho", "girassol", "hortela", "lavanda", "castanha", "semente", "tronco",
    "pomar", "horta", "campina",
];

const CATEGORIA_CEU: &[&str] = &[
    "aurora",
    "estrela",
    "arcoiris",
    "cometa",
    "galaxia",
    "orbita",
    "planeta",
    "eclipse",
    "constelacao",
    "meteorito",
    "saturno",
    "mercurio",
    "venus",
    "marte",
    "jupiter",
    "netuno",
    "plutao",
    "boreal",
    "austral",
    "pulsar",
];

const CATEGORIA_ANIMAIS: &[&str] = &[
    "tigre",
    "aguia",
    "lobo",
    "urso",
    "raposa",
    "coruja",
    "falcao",
    "pantera",
    "dragao",
    "beijaflor",
    "golfinho",
    "lontra",
    "cervo",
    "javali",
    "lebre",
    "esquilo",
    "gaivota",
    "andorinha",
    "faisao",
    "narval",
];

const CATEGORIA_OBJETOS: &[&str] = &[
    "ancora", "bussola", "cabana", "torre", "castelo", "janela", "cadeado", "escudo", "lanterna",
    "martelo", "bigorna", "mosaico", "ceramica", "quartzo", "ametista", "turquesa", "tapete",
    "tecido", "veludo", "artefato",
];

const CATEGORIA_IDEIAS: &[&str] = &[
    "harmonia",
    "serenidade",
    "equilibrio",
    "destino",
    "jornada",
    "legado",
    "memoria",
    "melodia",
    "sonata",
    "poesia",
    "fabula",
    "mito",
    "enigma",
    "simbolo",
    "codigo",
    "sentinela",
    "guardiao",
    "refugio",
    "horizonte",
    "claridade",
];

const CATEGORIAS_MEMORAVEIS: &[&[&str]] = &[
    PALAVRAS_CURTAS,
    CATEGORIA_NATUREZA,
    CATEGORIA_CEU,
    CATEGORIA_ANIMAIS,
    CATEGORIA_OBJETOS,
    CATEGORIA_IDEIAS,
];

const PALAVRAS: &[&str] = &[
    "sol",
    "lua",
    "mar",
    "rio",
    "flor",
    "ceu",
    "neve",
    "fogo",
    "terra",
    "vento",
    "pedra",
    "nuvem",
    "folha",
    "chuva",
    "onda",
    "bosque",
    "campo",
    "serra",
    "vale",
    "lago",
    "praia",
    "selva",
    "aurora",
    "estrela",
    "jardim",
    "planta",
    "fruta",
    "passaro",
    "montanha",
    "oceano",
    "cachoeira",
    "floresta",
    "relampago",
    "tempestade",
    "arcoiris",
    "cristal",
    "diamante",
    "esmeralda",
    "safira",
    "rubi",
    "agata",
    "coral",
    "perola",
    "ambar",
    "tigre",
    "aguia",
    "lobo",
    "urso",
    "gato",
    "leao",
    "raposa",
    "coruja",
    "falcao",
    "pantera",
    "cobra",
    "dragao",
    "amanhecer",
    "anoitecer",
    "brisa",
    "orvalho",
    "girassol",
    "hortela",
    "lavanda",
    "canela",
    "baunilha",
    "cafe",
    "cacau",
    "mel",
    "avela",
    "castanha",
    "amendoa",
    "trigo",
    "centeio",
    "aveia",
    "semente",
    "raiz",
    "petala",
    "ramo",
    "tronco",
    "vinha",
    "pomar",
    "horta",
    "manancial",
    "laguna",
    "delta",
    "enseada",
    "ilha",
    "arquipelago",
    "planicie",
    "colina",
    "penhasco",
    "caverna",
    "gruta",
    "deserto",
    "oasis",
    "moncao",
    "trovao",
    "neblina",
    "geada",
    "aurora",
    "cometa",
    "galaxia",
    "orbita",
    "planeta",
    "saturno",
    "mercurio",
    "venus",
    "marte",
    "jupiter",
    "netuno",
    "plutao",
    "eclipse",
    "constelacao",
    "meteorito",
    "farol",
    "tapete",
    "ancora",
    "timoneiro",
    "marujo",
    "ponte",
    "estrada",
    "trilha",
    "atalho",
    "mapa",
    "bussola",
    "abrigo",
    "cabana",
    "torre",
    "fortaleza",
    "castelo",
    "janela",
    "varanda",
    "telhado",
    "chave",
    "cadeado",
    "escudo",
    "lanterna",
    "tocha",
    "martelo",
    "bigorna",
    "flecha",
    "arco",
    "vela",
    "tecido",
    "linho",
    "seda",
    "veludo",
    "algodao",
    "mosaico",
    "ceramica",
    "granito",
    "basalto",
    "quartzo",
    "jade",
    "opala",
    "topazio",
    "turquesa",
    "ametista",
    "violeta",
    "anil",
    "ambarina",
    "escarlate",
    "dourado",
    "prateado",
    "cobre",
    "bronze",
    "ferro",
    "aco",
    "nobreza",
    "bravura",
    "calma",
    "harmonia",
    "serenidade",
    "equilibrio",
    "destino",
    "jornada",
    "legado",
    "memoria",
    "mistura",
    "ritmo",
    "melodia",
    "acorde",
    "sonata",
    "poesia",
    "verso",
    "conto",
    "fabula",
    "lenda",
    "mito",
    "saga",
    "enigma",
    "runa",
    "simbolo",
    "codigo",
    "sinal",
    "pulsar",
    "faisca",
    "chama",
    "brasas",
    "fornalha",
    "moinho",
    "atelier",
    "oficina",
    "laboratorio",
    "engenho",
    "artefato",
    "reliquia",
    "tesouro",
    "diametro",
    "vetor",
    "pixel",
    "nucleo",
    "pulso",
    "modulo",
    "circuito",
    "portal",
    "sentinela",
    "guardiao",
    "vigia",
    "reserva",
    "refugio",
    "mirante",
    "horizonte",
    "claridade",
    "sombra",
    "luminaria",
    "estreito",
    "serpente",
    "beijaflor",
    "golfinho",
    "lontra",
    "texugo",
    "cervo",
    "javali",
    "lebre",
    "esquilo",
    "rouxinol",
    "pardal",
    "gaivota",
    "andorinha",
    "garoa",
    "temporo",
    "arvoredo",
    "campina",
    "fiorde",
    "mangue",
    "restinga",
    "duna",
    "estalactite",
    "estalagmite",
    "faisao",
    "lince",
    "orca",
    "narval",
    "boreal",
    "austral",
    "zenite",
    "nadir",
];

#[derive(Deserialize)]
struct GerarRequest {
    tipo: String,
    tamanho: Option<u32>,
    maiusculas: Option<bool>,
    numeros: Option<bool>,
    simbolos: Option<bool>,
    excluir_ambiguos: Option<bool>,
    num_palavras: Option<u32>,
    separador: Option<String>,
    capitalizar: Option<bool>,
    incluir_numero: Option<bool>,
    tamanho_pin: Option<u32>,
}

#[derive(Serialize)]
struct GerarResponse {
    senha: String,
    forca: ForcaInfo,
}

#[derive(Serialize)]
struct ForcaInfo {
    pontuacao: u32,
    rotulo: String,
    cor: String,
    porcentagem: u32,
    tempo_quebra: String,
}

#[derive(Deserialize)]
struct AvaliarRequest {
    senha: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    erro: String,
}

struct RateLimitState {
    clients: HashMap<String, ClientRateLimitEntry>,
}

struct ClientRateLimitEntry {
    window_start: Instant,
    count: u32,
}

struct RateLimitHeaders {
    limit: u32,
    remaining: u32,
    retry_after: Option<u64>,
}

enum GerarConfig {
    Aleatoria {
        tamanho: u32,
        maiusculas: bool,
        numeros: bool,
        simbolos: bool,
        excluir_ambiguos: bool,
    },
    Memoravel {
        num_palavras: u32,
        separador: String,
        capitalizar: bool,
        incluir_numero: bool,
    },
    Pin {
        tamanho_pin: u32,
    },
}

#[tokio::main]
async fn main() {
    log_event("info", "startup", "Inicializando servidor");

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/robots.txt", get(serve_robots_txt))
        .route("/sitemap.xml", get(serve_sitemap_xml))
        .route("/api/gerar", post(api_gerar))
        .route("/api/avaliar", post(api_avaliar))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(security_headers_middleware));

    let port = listen_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log_event(
        "info",
        "startup",
        &format!("Servidor rodando em http://0.0.0.0:{port}"),
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    let html = include_str!("../static/index.html")
        .replace("__SITE_NAME__", &site_name())
        .replace("__SITE_DESCRIPTION__", DEFAULT_SITE_DESCRIPTION)
        .replace("__SITE_URL__", &site_url());

    Html(html)
}

async fn serve_robots_txt() -> impl IntoResponse {
    let robots = format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
        site_url()
    );

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}

async fn serve_sitemap_xml() -> impl IntoResponse {
    let url = site_url();
    let sitemap = format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
            "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">",
            "<url>",
            "<loc>{url}/</loc>",
            "<changefreq>weekly</changefreq>",
            "<priority>1.0</priority>",
            "</url>",
            "</urlset>"
        ),
        url = url
    );

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        sitemap,
    )
}

fn site_name() -> String {
    env::var("SITE_NAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_SITE_NAME.to_string())
}

fn site_domain() -> String {
    env::var("DOMAIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "example.com".to_string())
}

fn site_url() -> String {
    env::var("SITE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("RENDER_EXTERNAL_URL")
                .ok()
                .map(|value| value.trim().trim_end_matches('/').to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| format!("https://{}", site_domain()))
}

fn listen_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT)
}

async fn api_gerar(headers: HeaderMap, body: Bytes) -> Response {
    if let Err(response) = validate_json_content_type(&headers) {
        log_security_event(
            "content_type_invalido",
            "Content-Type diferente de application/json",
        );
        return response;
    }

    let rate_limit = match enforce_rate_limit(&headers) {
        Ok(info) => info,
        Err(response) => return response,
    };

    let req: GerarRequest = match parse_json_body(body) {
        Ok(req) => req,
        Err(response) => {
            log_security_event("json_invalido", "Falha ao interpretar JSON em /api/gerar");
            return response;
        }
    };

    let mut response = match validar_gerar_request(&req) {
        Ok(config) => {
            let senha = match config {
                GerarConfig::Aleatoria {
                    tamanho,
                    maiusculas,
                    numeros,
                    simbolos,
                    excluir_ambiguos,
                } => gerar_aleatoria(tamanho, maiusculas, numeros, simbolos, excluir_ambiguos),
                GerarConfig::Memoravel {
                    num_palavras,
                    separador,
                    capitalizar,
                    incluir_numero,
                } => gerar_memoravel(num_palavras, &separador, capitalizar, incluir_numero),
                GerarConfig::Pin { tamanho_pin } => gerar_pin(tamanho_pin),
            };

            let forca = avaliar_forca(&senha);
            (StatusCode::OK, axum::Json(GerarResponse { senha, forca })).into_response()
        }
        Err(message) => {
            log_security_event("validacao_bloqueada", &message);
            (
                StatusCode::BAD_REQUEST,
                axum::Json(ErrorResponse { erro: message }),
            )
                .into_response()
        }
    };

    apply_rate_limit_headers(response.headers_mut(), &rate_limit);
    response
}

async fn api_avaliar(headers: HeaderMap, body: Bytes) -> Response {
    if let Err(response) = validate_json_content_type(&headers) {
        log_security_event(
            "content_type_invalido",
            "Content-Type diferente de application/json",
        );
        return response;
    }

    let rate_limit = match enforce_rate_limit(&headers) {
        Ok(info) => info,
        Err(response) => return response,
    };

    let req: AvaliarRequest = match parse_json_body(body) {
        Ok(req) => req,
        Err(response) => {
            log_security_event("json_invalido", "Falha ao interpretar JSON em /api/avaliar");
            return response;
        }
    };

    let mut response = if req.senha.is_empty() || req.senha.len() > 256 {
        log_security_event(
            "avaliacao_bloqueada",
            "Senha para avaliacao deve ter entre 1 e 256 caracteres",
        );
        (
            StatusCode::BAD_REQUEST,
            axum::Json(ErrorResponse {
                erro: "Senha para avaliacao deve ter entre 1 e 256 caracteres".to_string(),
            }),
        )
            .into_response()
    } else {
        (StatusCode::OK, axum::Json(avaliar_forca(&req.senha))).into_response()
    };

    apply_rate_limit_headers(response.headers_mut(), &rate_limit);
    response
}

fn validar_gerar_request(req: &GerarRequest) -> Result<GerarConfig, String> {
    match req.tipo.as_str() {
        "aleatorio" => {
            let tamanho = req.tamanho.unwrap_or(16);
            if !(4..=64).contains(&tamanho) {
                return Err("Comprimento deve estar entre 4 e 64".to_string());
            }

            let maiusculas = req.maiusculas.unwrap_or(true);
            let numeros = req.numeros.unwrap_or(true);
            let simbolos = req.simbolos.unwrap_or(true);

            if !maiusculas && !numeros && !simbolos {
                return Err(
                    "Selecione pelo menos uma opcao adicional alem das minusculas".to_string(),
                );
            }

            Ok(GerarConfig::Aleatoria {
                tamanho,
                maiusculas,
                numeros,
                simbolos,
                excluir_ambiguos: req.excluir_ambiguos.unwrap_or(false),
            })
        }
        "memoravel" => {
            let num_palavras = req.num_palavras.unwrap_or(4);
            if !(2..=8).contains(&num_palavras) {
                return Err("Quantidade de palavras deve estar entre 2 e 8".to_string());
            }

            let separador = req.separador.clone().unwrap_or_else(|| "-".to_string());
            if !matches!(separador.as_str(), "-" | "." | "_" | " ") {
                return Err("Separador invalido".to_string());
            }

            Ok(GerarConfig::Memoravel {
                num_palavras,
                separador,
                capitalizar: req.capitalizar.unwrap_or(true),
                incluir_numero: req.incluir_numero.unwrap_or(true),
            })
        }
        "pin" => {
            let tamanho_pin = req.tamanho_pin.unwrap_or(6);
            if !(4..=12).contains(&tamanho_pin) {
                return Err("PIN deve ter entre 4 e 12 digitos".to_string());
            }

            Ok(GerarConfig::Pin { tamanho_pin })
        }
        _ => Err("Tipo de senha invalido".to_string()),
    }
}

fn validate_json_content_type(headers: &HeaderMap) -> Result<(), Response> {
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .map(|item| item.trim().eq_ignore_ascii_case("application/json"))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    if is_json {
        return Ok(());
    }

    Err((
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        axum::Json(ErrorResponse {
            erro: "Content-Type deve ser application/json".to_string(),
        }),
    )
        .into_response())
}

fn parse_json_body<T: DeserializeOwned>(body: Bytes) -> Result<T, Response> {
    serde_json::from_slice::<T>(&body).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(ErrorResponse {
                erro: "JSON invalido".to_string(),
            }),
        )
            .into_response()
    })
}

fn enforce_rate_limit(headers: &HeaderMap) -> Result<RateLimitHeaders, Response> {
    let now = Instant::now();
    let window = Duration::from_secs(RATE_LIMIT_WINDOW_SECS);
    let client_id = resolve_client_id(headers);
    let mut state = rate_limit_store().lock().unwrap();

    state
        .clients
        .retain(|_, entry| now.duration_since(entry.window_start) < window);

    let entry = state
        .clients
        .entry(client_id)
        .or_insert_with(|| ClientRateLimitEntry {
            window_start: now,
            count: 0,
        });

    if now.duration_since(entry.window_start) >= window {
        entry.window_start = now;
        entry.count = 0;
    }

    if entry.count >= RATE_LIMIT_MAX_REQUESTS {
        let retry_after = window
            .saturating_sub(now.duration_since(entry.window_start))
            .as_secs()
            .max(1);
        log_security_event(
            "rate_limit",
            "Muitas requisicoes no intervalo de 60 segundos",
        );
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(ErrorResponse {
                erro: "Muitas requisicoes. Tente novamente em instantes".to_string(),
            }),
        )
            .into_response();
        response.headers_mut().insert(
            header::RETRY_AFTER,
            HeaderValue::from_str(&retry_after.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("60")),
        );
        apply_rate_limit_headers(
            response.headers_mut(),
            &RateLimitHeaders {
                limit: RATE_LIMIT_MAX_REQUESTS,
                remaining: 0,
                retry_after: Some(retry_after),
            },
        );
        return Err(response);
    }

    entry.count += 1;
    Ok(RateLimitHeaders {
        limit: RATE_LIMIT_MAX_REQUESTS,
        remaining: RATE_LIMIT_MAX_REQUESTS.saturating_sub(entry.count),
        retry_after: None,
    })
}

fn rate_limit_store() -> &'static Mutex<RateLimitState> {
    static STORE: OnceLock<Mutex<RateLimitState>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(RateLimitState {
            clients: HashMap::new(),
        })
    })
}

fn resolve_client_id(headers: &HeaderMap) -> String {
    if let Some(client_ip) = headers
        .get("x-client-ip")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
    {
        return client_ip.trim().to_string();
    }

    if let Some(forwarded_for) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
    {
        let clients: Vec<&str> = forwarded_for
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .collect();

        if !clients.is_empty() {
            let client_index = clients.len().saturating_sub(TRUSTED_PROXY_HOPS + 1);
            return clients[client_index].to_string();
        }
    }

    headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn apply_rate_limit_headers(headers: &mut HeaderMap, rate_limit: &RateLimitHeaders) {
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_str(&rate_limit.limit.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("120")),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_str(&rate_limit.remaining.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );

    if let Some(retry_after) = rate_limit.retry_after {
        headers.insert(
            header::RETRY_AFTER,
            HeaderValue::from_str(&retry_after.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("60")),
        );
    }
}

async fn request_logging_middleware(request: axum::extract::Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let client_ip = resolve_client_id(request.headers());
    let request_id = next_request_id();
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_string();
    let forwarded_for = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_string();
    let forwarded_proto = request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_string();

    let mut response = next.run(request).await;
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id)
            .unwrap_or_else(|_| HeaderValue::from_static("invalid-request-id")),
    );
    log_request_event(
        &method,
        &path,
        response.status().as_u16(),
        &request_id,
        &client_ip,
        &origin,
        &forwarded_for,
        &forwarded_proto,
    );
    response
}

async fn security_headers_middleware(request: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; manifest-src 'self'; require-trusted-types-for 'script'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));

    response
}

fn log_security_event(event: &str, detail: &str) {
    log_event("security", event, detail);
}

fn log_request_event(
    method: &Method,
    path: &str,
    status: u16,
    request_id: &str,
    client_ip: &str,
    origin: &str,
    forwarded_for: &str,
    forwarded_proto: &str,
) {
    let detail = format!(
        "request_id={request_id} method={method} path={path} status={status} client_ip={client_ip} origin={origin} forwarded_for={forwarded_for} forwarded_proto={forwarded_proto}"
    );
    log_event("access", "request", &detail);
}

fn log_event(level: &str, event: &str, detail: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    eprintln!(
        "{{\"ts\":{timestamp},\"level\":\"{level}\",\"event\":\"{event}\",\"detail\":\"{detail}\"}}"
    );
}

fn next_request_id() -> String {
    static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    format!("req-{timestamp:x}-{counter:x}")
}

fn gerar_aleatoria(
    tamanho: u32,
    usar_maiusculas: bool,
    usar_numeros: bool,
    usar_simbolos: bool,
    excluir_ambiguos: bool,
) -> String {
    let mut rng = OsRng;
    let mut pool = String::from(MINUSCULAS);
    let mut obrigatorios: Vec<char> = Vec::new();

    let min_chars: Vec<char> = MINUSCULAS.chars().collect();
    obrigatorios.push(min_chars[rng.gen_range(0..min_chars.len())]);

    if usar_maiusculas {
        pool.push_str(MAIUSCULAS);
        let chars: Vec<char> = MAIUSCULAS.chars().collect();
        obrigatorios.push(chars[rng.gen_range(0..chars.len())]);
    }
    if usar_numeros {
        pool.push_str(NUMEROS);
        let chars: Vec<char> = NUMEROS.chars().collect();
        obrigatorios.push(chars[rng.gen_range(0..chars.len())]);
    }
    if usar_simbolos {
        pool.push_str(SIMBOLOS);
        let chars: Vec<char> = SIMBOLOS.chars().collect();
        obrigatorios.push(chars[rng.gen_range(0..chars.len())]);
    }

    if excluir_ambiguos {
        pool = pool.chars().filter(|c| !AMBIGUOS.contains(*c)).collect();
        obrigatorios.retain(|c| !AMBIGUOS.contains(*c));
    }

    let pool_chars: Vec<char> = pool.chars().collect();
    let tam = tamanho as usize;

    let mut senha: Vec<char> = Vec::with_capacity(tam);
    senha.extend(obrigatorios.iter().copied());
    while senha.len() < tam {
        senha.push(pool_chars[rng.gen_range(0..pool_chars.len())]);
    }

    senha.truncate(tam);
    senha.shuffle(&mut rng);
    senha.into_iter().collect()
}

fn gerar_memoravel(
    num_palavras: u32,
    separador: &str,
    capitalizar: bool,
    incluir_numero: bool,
) -> String {
    let mut rng = OsRng;
    let mut palavras: Vec<String> = Vec::new();
    let mut categorias: Vec<&[&str]> = CATEGORIAS_MEMORAVEIS.to_vec();
    let mut raizes_usadas: HashSet<String> = HashSet::new();

    categorias.shuffle(&mut rng);

    for categoria in categorias {
        if palavras.len() >= num_palavras as usize {
            break;
        }

        if let Some(palavra) = escolher_palavra_distinta(categoria, &raizes_usadas, &mut rng) {
            raizes_usadas.insert(raiz_palavra(palavra));
            palavras.push(formatar_palavra_memoravel(palavra, capitalizar));
        }
    }

    while palavras.len() < num_palavras as usize {
        if let Some(palavra) = escolher_palavra_distinta(PALAVRAS, &raizes_usadas, &mut rng) {
            raizes_usadas.insert(raiz_palavra(palavra));
            palavras.push(formatar_palavra_memoravel(palavra, capitalizar));
        } else {
            break;
        }
    }

    if incluir_numero {
        let numero = rng.gen_range(0..100);
        let pos = rng.gen_range(0..=palavras.len());
        palavras.insert(pos, numero.to_string());
    }

    palavras.join(separador)
}

fn gerar_pin(tamanho: u32) -> String {
    let mut rng = OsRng;
    (0..tamanho)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}

fn escolher_palavra_distinta<'a>(
    pool: &'a [&'a str],
    raizes_usadas: &HashSet<String>,
    rng: &mut OsRng,
) -> Option<&'a str> {
    let mut candidatas: Vec<&str> = pool
        .iter()
        .copied()
        .filter(|palavra| !raizes_usadas.contains(&raiz_palavra(palavra)))
        .collect();

    if candidatas.is_empty() {
        return None;
    }

    candidatas.shuffle(rng);
    candidatas.into_iter().next()
}

fn formatar_palavra_memoravel(palavra: &str, capitalizar: bool) -> String {
    if capitalizar {
        capitalizar_palavra(palavra)
    } else {
        palavra.to_string()
    }
}

fn raiz_palavra(palavra: &str) -> String {
    palavra.chars().take(4).collect()
}

fn capitalizar_palavra(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn avaliar_forca(senha: &str) -> ForcaInfo {
    let len = senha.len();
    let eh_pin = senha.chars().all(|c| c.is_ascii_digit());
    let tem_minuscula = senha.chars().any(|c| c.is_ascii_lowercase());
    let tem_maiuscula = senha.chars().any(|c| c.is_ascii_uppercase());
    let tem_numero = senha.chars().any(|c| c.is_ascii_digit());
    let tem_simbolo = senha.chars().any(|c| !c.is_alphanumeric());

    let palavras_memoraveis: Vec<&str> = senha
        .split(|c: char| !c.is_alphabetic())
        .filter(|parte| !parte.is_empty())
        .collect();
    let usa_apenas_componentes_memoraveis = senha
        .chars()
        .all(|c| c.is_alphabetic() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_' | ' '));
    let eh_memoravel = palavras_memoraveis.len() >= 2
        && senha.chars().any(|c| matches!(c, '-' | '.' | '_' | ' '))
        && usa_apenas_componentes_memoraveis;

    let mut charset: f64 = 0.0;
    if tem_minuscula {
        charset += 26.0;
    }
    if tem_maiuscula {
        charset += 26.0;
    }
    if tem_numero {
        charset += 10.0;
    }
    if tem_simbolo {
        charset += 32.0;
    }
    if charset == 0.0 {
        charset = 10.0;
    }

    let mut bits_entropia = if eh_pin {
        (len as f64) * 10.0_f64.log2()
    } else if eh_memoravel {
        let base_palavras = 11.0_f64.max(PALAVRAS.len() as f64);
        let separadores = senha
            .chars()
            .filter(|c| matches!(c, '-' | '.' | '_' | ' '))
            .count();
        let separador_bits = if separadores > 0 {
            (separadores as f64) * 4.0_f64.log2()
        } else {
            0.0
        };
        let numero_bits = if tem_numero {
            10.0_f64.log2() * 2.0
        } else {
            0.0
        };
        let capitalizacao_bits = if tem_maiuscula {
            palavras_memoraveis.len() as f64
        } else {
            0.0
        };

        (palavras_memoraveis.len() as f64) * base_palavras.log2()
            + separador_bits
            + numero_bits
            + capitalizacao_bits
    } else {
        (len as f64) * charset.log2()
    };

    let chars: Vec<char> = senha.chars().collect();
    let mut repeticoes = 0u32;
    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            repeticoes += 1;
        }
    }

    let sequencias = chars
        .windows(3)
        .filter(|janela| {
            let a = janela[0] as i32;
            let b = janela[1] as i32;
            let c = janela[2] as i32;
            (b - a == 1 && c - b == 1) || (a - b == 1 && b - c == 1)
        })
        .count() as u32;

    let raizes_repetidas = if eh_memoravel {
        let mut vistas = HashSet::new();
        let mut repetidas = 0u32;
        for palavra in &palavras_memoraveis {
            let raiz = raiz_palavra(&palavra.to_lowercase());
            if !vistas.insert(raiz) {
                repetidas += 1;
            }
        }
        repetidas
    } else {
        0
    };

    bits_entropia -= repeticoes as f64 * 2.5;
    bits_entropia -= sequencias as f64 * 4.0;
    bits_entropia -= raizes_repetidas as f64 * 6.0;

    if eh_pin {
        let todos_iguais = chars.windows(2).all(|janela| janela[0] == janela[1]);
        if todos_iguais {
            bits_entropia -= 12.0;
        }
    }

    let bits_entropia = bits_entropia.max(0.0);
    let pontuacao = if eh_pin {
        ((bits_entropia / 1.1).round() as u32).min(100)
    } else if eh_memoravel {
        ((bits_entropia / 0.65).round() as u32).min(100)
    } else {
        ((bits_entropia / 1.2).round() as u32).min(100)
    };

    let segundos = charset.powf(len as f64) / 1_000_000_000.0;

    let muito_forte_minimo = if eh_pin {
        90
    } else if eh_memoravel {
        85
    } else {
        80
    };

    let (rotulo, cor, porcentagem) = if pontuacao <= 25 {
        ("Muito Fraca", "#ef4444", pontuacao)
    } else if pontuacao < 45 {
        ("Fraca", "#f97316", pontuacao)
    } else if pontuacao < 65 {
        ("Razoavel", "#eab308", pontuacao)
    } else if pontuacao < muito_forte_minimo {
        ("Forte", "#22c55e", pontuacao)
    } else {
        ("Muito Forte", "#06b6d4", 100)
    };

    ForcaInfo {
        pontuacao,
        rotulo: rotulo.to_string(),
        cor: cor.to_string(),
        porcentagem,
        tempo_quebra: formatar_tempo(segundos),
    }
}

fn formatar_tempo(segundos: f64) -> String {
    if segundos < 1.0 {
        "Instantaneo".to_string()
    } else if segundos < 60.0 {
        format!("{segundos:.0} segundos")
    } else if segundos < 3600.0 {
        format!("{:.0} minutos", segundos / 60.0)
    } else if segundos < 86400.0 {
        format!("{:.0} horas", segundos / 3600.0)
    } else if segundos < 31_536_000.0 {
        format!("{:.0} dias", segundos / 86400.0)
    } else if segundos < 31_536_000.0 * 1000.0 {
        format!("{:.0} anos", segundos / 31_536_000.0)
    } else if segundos < 31_536_000.0 * 1_000_000.0 {
        format!("{:.0} mil anos", segundos / (31_536_000.0 * 1000.0))
    } else {
        "Muito tempo".to_string()
    }
}
