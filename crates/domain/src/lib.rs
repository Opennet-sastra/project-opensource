//! Domain types and traits (no I/O, no heavy dependencies).

#[derive(Debug)]
pub enum DomainError {
    Network(String),
    Auth(String),
    Parse(String),
    Captcha(String),
    Other(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::Auth(msg) => write!(f, "Auth error: {msg}"),
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::Captcha(msg) => write!(f, "Captcha error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DomainError {}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Credentials {
    pub reg_number: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Session {
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CaptchaChallenge {
    pub session_id: String,
    pub data_uri: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StudentProfile {
    pub student_id: String,
    pub name: String,
    pub program: String,
    pub institution: String,
    pub semester: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AttendanceRecord {
    pub code: String,
    pub description: String,
    pub total: u32,
    pub present: u32,
    pub absent: u32,
    pub percentage: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InternalMark {
    pub code: String,
    pub description: String,
    pub component: String,
    pub marks: String,
    pub max_marks: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GradeRecord {
    pub semester: u32,
    pub month_year: String,
    pub code: String,
    pub description: String,
    pub cia: String,
    pub credits: u32,
    pub grade: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GpaSummary {
    pub cgpa: Option<String>,
    pub sgpa_list: Vec<(u32, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ExamScheduleItem {
    pub date: String,
    pub time: String,
    pub code: String,
    pub description: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct HostelDetails {
    pub hostel_name: String,
    pub room_no: String,
    pub allocation_date: String,
    pub mess_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AbsentHourRecord {
    pub date: String,
    pub code: String,
    pub description: String,
    pub hour: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CourseHistoryItem {
    pub code: String,
    pub description: String,
    pub semester: String,
    pub credits: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PortalData {
    pub profile: Option<StudentProfile>,
    pub attendance: Vec<AttendanceRecord>,
    pub marks: Vec<InternalMark>,
    pub grades: Vec<GradeRecord>,
    pub gpa: Option<GpaSummary>,
    pub exams: Vec<ExamScheduleItem>,
    pub hostel: Option<HostelDetails>,
    pub absent_hours: Vec<AbsentHourRecord>,
    pub course_history: Vec<CourseHistoryItem>,
}

/// Abstract contract for reading data from a portal source.
pub trait PortalSource: Send + Sync {
    fn fetch_captcha(
        &self,
    ) -> impl std::future::Future<Output = Result<CaptchaChallenge, DomainError>> + Send;
    fn login(
        &self,
        creds: &Credentials,
        captcha_code: &str,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<Session, DomainError>> + Send;
    fn fetch_portal_data(
        &self,
        session: &Session,
    ) -> impl std::future::Future<Output = Result<PortalData, DomainError>> + Send;
}
