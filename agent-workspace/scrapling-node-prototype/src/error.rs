use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ScraplingError {
    #[error("SSRF blocked: {url} resolves to private range")]
    SsrfBlocked { url: String },
    
    #[error("HTTP error {status}: {message}")]
    HttpError { status: u16, message: String },
    
    #[error("Extraction failed: {reason}")]
    ExtractionFailed { reason: String },
    
    #[error("Robots.txt disallowed: {url}")]
    RobotsBlocked { url: String },
    
    #[error("Rate limited: retry after {seconds}s")]
    RateLimited { seconds: u64 },
    
    #[error("Max retries exceeded: {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },
}
