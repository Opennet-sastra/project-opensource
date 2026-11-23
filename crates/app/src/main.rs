#![allow(non_snake_case)]

use dioxus::prelude::*;
use storage::Storage;
use sync::{CaptchaChallenge, Credentials, PortalData, SyncService};

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Profile,
    Attendance,
    Marks,
    Grades,
    Exams,
    Hostel,
    Absent,
    Courses,
}

#[component]
fn App() -> Element {
    let storage = use_hook(Storage::new);
    let sync_service = use_hook(|| SyncService::new(storage.clone()));

    let mut reg_no = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut captcha_input = use_signal(String::new);
    let mut captcha_challenge = use_signal(|| None::<CaptchaChallenge>);
    let mut is_logged_in = use_signal(|| false);
    let mut active_tab = use_signal(|| Tab::Profile);
    let mut status_msg = use_signal(|| "Ready".to_string());
    let mut is_loading = use_signal(|| false);
    let mut portal_data = use_signal(PortalData::default);

    // Initial captcha load
    let service_for_load = sync_service.clone();
    use_effect(move || {
        let svc = service_for_load.clone();
        spawn(async move {
            is_loading.set(true);
            status_msg.set("Loading captcha…".to_string());
            match svc.fetch_captcha().await {
                Ok(ch) => {
                    // Try auto-solving with OCR if available
                    if let Some(ocr_text) = svc.try_solve_captcha(&ch.data_uri).await {
                        captcha_input.set(ocr_text);
                        status_msg.set("Captcha auto-filled by OCR".to_string());
                    } else {
                        status_msg.set("Enter the captcha below".to_string());
                    }
                    captcha_challenge.set(Some(ch));
                }
                Err(err) => {
                    status_msg.set(format!("Failed to load captcha: {err}"));
                }
            }
            is_loading.set(false);
        });
    });

    let reload_captcha = {
        let svc = sync_service.clone();
        move |_| {
            let svc = svc.clone();
            spawn(async move {
                is_loading.set(true);
                status_msg.set("Refreshing captcha…".to_string());
                captcha_input.set(String::new());
                match svc.fetch_captcha().await {
                    Ok(ch) => {
                        if let Some(ocr_text) = svc.try_solve_captcha(&ch.data_uri).await {
                            captcha_input.set(ocr_text);
                            status_msg.set("Captcha auto-filled by OCR".to_string());
                        } else {
                            status_msg.set("Enter the captcha below".to_string());
                        }
                        captcha_challenge.set(Some(ch));
                    }
                    Err(err) => {
                        status_msg.set(format!("Captcha error: {err}"));
                    }
                }
                is_loading.set(false);
            });
        }
    };

    let on_login = {
        let svc = sync_service.clone();
        move |_| {
            let svc = svc.clone();
            let reg = reg_no().trim().to_string();
            let pwd = password().trim().to_string();
            let cap = captcha_input().trim().to_string();
            let ch = captcha_challenge();

            if reg.is_empty() || pwd.is_empty() || cap.is_empty() {
                status_msg.set("Please fill in all fields".to_string());
                return;
            }

            let Some(challenge) = ch else {
                status_msg.set("No captcha loaded. Refresh captcha first.".to_string());
                return;
            };

            spawn(async move {
                is_loading.set(true);
                status_msg.set("Logging in and fetching data…".to_string());

                let creds = Credentials {
                    reg_number: reg,
                    password: pwd,
                };

                match svc
                    .login_and_sync(&creds, &cap, &challenge.session_id)
                    .await
                {
                    Ok(()) => {
                        portal_data.set(svc.storage().get_portal_data());
                        is_logged_in.set(true);
                        status_msg.set("Login successful!".to_string());
                    }
                    Err(err) => {
                        status_msg.set(format!("Login failed: {err}"));
                        // Refresh captcha on failure
                        if let Ok(new_ch) = svc.fetch_captcha().await {
                            captcha_input.set(String::new());
                            captcha_challenge.set(Some(new_ch));
                        }
                    }
                }
                is_loading.set(false);
            });
        }
    };

    let on_logout = {
        let storage = storage.clone();
        let svc = sync_service.clone();
        move |_| {
            storage.clear_session();
            is_logged_in.set(false);
            portal_data.set(PortalData::default());
            status_msg.set("Logged out".to_string());
            let svc = svc.clone();
            spawn(async move {
                if let Ok(ch) = svc.fetch_captcha().await {
                    captcha_input.set(String::new());
                    captcha_challenge.set(Some(ch));
                }
            });
        }
    };

    rsx! {
        div {
            style: "font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; \
                    background: #f4f6f8; min-height: 100vh; padding: 12px; color: #1a202c;",

            // Header
            div {
                style: "background: #1e3a8a; color: white; padding: 16px; border-radius: 8px; \
                        display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                h1 { style: "font-size: 1.25rem; margin: 0; font-weight: bold;", "SASTRA Portal" }
                if is_logged_in() {
                    button {
                        style: "background: #ef4444; color: white; border: none; padding: 6px 14px; \
                                border-radius: 6px; font-weight: 600; cursor: pointer;",
                        onclick: on_logout,
                        "Logout"
                    }
                }
            }

            // Status notification bar
            div {
                style: "background: #e2e8f0; padding: 8px 12px; border-radius: 6px; \
                        font-size: 0.875rem; margin-bottom: 12px; color: #334155;",
                "{status_msg()}"
            }

            if !is_logged_in() {
                // Login View
                div {
                    style: "background: white; padding: 20px; border-radius: 8px; \
                            box-shadow: 0 1px 3px rgba(0,0,0,0.1); max-width: 420px; margin: 0 auto;",

                    h2 { style: "font-size: 1.1rem; margin-top: 0; margin-bottom: 16px;", "Student Login" }

                    div { style: "margin-bottom: 12px;",
                        label { style: "display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 4px;", "Register Number" }
                        input {
                            style: "width: 100%; box-sizing: border-box; padding: 10px; border: 1px solid #cbd5e1; border-radius: 6px;",
                            r#type: "text",
                            placeholder: "e.g. 125000000",
                            value: "{reg_no()}",
                            oninput: move |e| reg_no.set(e.value()),
                        }
                    }

                    div { style: "margin-bottom: 12px;",
                        label { style: "display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 4px;", "Password" }
                        input {
                            style: "width: 100%; box-sizing: border-box; padding: 10px; border: 1px solid #cbd5e1; border-radius: 6px;",
                            r#type: "password",
                            placeholder: "Portal password",
                            value: "{password()}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }

                    // Captcha image container
                    div { style: "margin-bottom: 12px; text-align: center;",
                        label { style: "display: block; font-size: 0.85rem; font-weight: 600; margin-bottom: 6px; text-align: left;", "Captcha" }
                        if let Some(ch) = captcha_challenge() {
                            div {
                                style: "display: flex; align-items: center; justify-content: center; gap: 8px; margin-bottom: 8px;",
                                img {
                                    src: "{ch.data_uri}",
                                    style: "height: 48px; border: 1px solid #cbd5e1; border-radius: 4px;",
                                    alt: "Captcha",
                                }
                                button {
                                    style: "background: #f1f5f9; border: 1px solid #cbd5e1; padding: 8px 12px; \
                                            border-radius: 6px; cursor: pointer; font-size: 0.85rem;",
                                    onclick: reload_captcha,
                                    "↺ Reload"
                                }
                            }
                        } else {
                            p { style: "font-size: 0.85rem; color: #64748b;", "Loading captcha image…" }
                        }
                        input {
                            style: "width: 100%; box-sizing: border-box; padding: 10px; border: 1px solid #cbd5e1; border-radius: 6px;",
                            r#type: "text",
                            placeholder: "Enter letters shown above",
                            value: "{captcha_input()}",
                            oninput: move |e| captcha_input.set(e.value()),
                        }
                    }

                    button {
                        style: "width: 100%; background: #2563eb; color: white; padding: 12px; \
                                border: none; border-radius: 6px; font-size: 1rem; font-weight: 600; \
                                cursor: pointer; margin-top: 8px;",
                        disabled: is_loading(),
                        onclick: on_login,
                        if is_loading() { "Signing in…" } else { "Sign In" }
                    }
                }
            } else {
                // Portal Dashboard View
                div {
                    // Navigation tabs
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 16px;",
                        TabButton { title: "Profile", active: active_tab() == Tab::Profile, onclick: move |_| active_tab.set(Tab::Profile) }
                        TabButton { title: "Attendance", active: active_tab() == Tab::Attendance, onclick: move |_| active_tab.set(Tab::Attendance) }
                        TabButton { title: "Marks", active: active_tab() == Tab::Marks, onclick: move |_| active_tab.set(Tab::Marks) }
                        TabButton { title: "Grades", active: active_tab() == Tab::Grades, onclick: move |_| active_tab.set(Tab::Grades) }
                        TabButton { title: "Exams", active: active_tab() == Tab::Exams, onclick: move |_| active_tab.set(Tab::Exams) }
                        TabButton { title: "Hostel", active: active_tab() == Tab::Hostel, onclick: move |_| active_tab.set(Tab::Hostel) }
                        TabButton { title: "Absent", active: active_tab() == Tab::Absent, onclick: move |_| active_tab.set(Tab::Absent) }
                        TabButton { title: "Courses", active: active_tab() == Tab::Courses, onclick: move |_| active_tab.set(Tab::Courses) }
                    }

                    // Content Area
                    div {
                        style: "background: white; padding: 16px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);",

                        match active_tab() {
                            Tab::Profile => rsx! {
                                ProfileView { profile: portal_data().profile }
                            },
                            Tab::Attendance => rsx! {
                                AttendanceView { list: portal_data().attendance }
                            },
                            Tab::Marks => rsx! {
                                MarksView { list: portal_data().marks }
                            },
                            Tab::Grades => rsx! {
                                GradesView { gpa: portal_data().gpa, list: portal_data().grades }
                            },
                            Tab::Exams => rsx! {
                                ExamsView { list: portal_data().exams }
                            },
                            Tab::Hostel => rsx! {
                                HostelView { hostel: portal_data().hostel }
                            },
                            Tab::Absent => rsx! {
                                AbsentHoursView { list: portal_data().absent_hours }
                            },
                            Tab::Courses => rsx! {
                                CoursesView { list: portal_data().course_history }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TabButton(title: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let bg = if active { "#1e3a8a" } else { "#e2e8f0" };
    let fg = if active { "white" } else { "#1e293b" };
    rsx! {
        button {
            style: "background: {bg}; color: {fg}; border: none; padding: 8px 12px; \
                    border-radius: 6px; font-size: 0.85rem; font-weight: 600; cursor: pointer;",
            onclick: move |e| onclick.call(e),
            "{title}"
        }
    }
}

#[component]
fn ProfileView(profile: Option<sync::StudentProfile>) -> Element {
    if let Some(p) = profile {
        rsx! {
            div {
                h2 { style: "font-size: 1.1rem; margin-top: 0;", "Student Profile" }
                InfoRow { label: "Student ID", value: p.student_id }
                InfoRow { label: "Name", value: p.name }
                InfoRow { label: "Program", value: p.program }
                InfoRow { label: "Institution", value: p.institution }
                InfoRow { label: "Semester", value: p.semester }
            }
        }
    } else {
        rsx! { p { "No profile information available." } }
    }
}

#[component]
fn InfoRow(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #f1f5f9;",
            span { style: "color: #64748b; font-weight: 500;", "{label}" }
            span { style: "font-weight: 600;", "{value}" }
        }
    }
}

#[component]
fn AttendanceView(list: Vec<sync::AttendanceRecord>) -> Element {
    rsx! {
        div {
            h2 { style: "font-size: 1.1rem; margin-top: 0;", "Attendance (Course-wise)" }
            if list.is_empty() {
                p { "No attendance records found." }
            } else {
                div {
                    style: "overflow-x: auto;",
                    table {
                        style: "width: 100%; border-collapse: collapse; font-size: 0.85rem;",
                        thead {
                            tr {
                                style: "background: #f8fafc; text-align: left;",
                                th { style: "padding: 8px; border-bottom: 2px solid #e2e8f0;", "Course" }
                                th { style: "padding: 8px; border-bottom: 2px solid #e2e8f0; text-align: center;", "Attended" }
                                th { style: "padding: 8px; border-bottom: 2px solid #e2e8f0; text-align: center;", "%" }
                            }
                        }
                        tbody {
                            for item in list {
                                tr {
                                    style: "border-bottom: 1px solid #f1f5f9;",
                                    td { style: "padding: 8px;",
                                        div { style: "font-weight: 600;", "{item.code}" }
                                        div { style: "color: #64748b; font-size: 0.75rem;", "{item.description}" }
                                    }
                                    td { style: "padding: 8px; text-align: center;", "{item.present} / {item.total}" }
                                    td {
                                        style: "padding: 8px; text-align: center; font-weight: bold; \
                                                color: if item.percentage < 75 { \"#dc2626\" } else { \"#16a34a\" };",
                                        "{item.percentage}%"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MarksView(list: Vec<sync::InternalMark>) -> Element {
    rsx! {
        div {
            h2 { style: "font-size: 1.1rem; margin-top: 0;", "Internal Marks" }
            if list.is_empty() {
                p { "No marks recorded." }
            } else {
                div {
                    style: "overflow-x: auto;",
                    table {
                        style: "width: 100%; border-collapse: collapse; font-size: 0.85rem;",
                        thead {
                            tr {
                                style: "background: #f8fafc; text-align: left;",
                                th { style: "padding: 8px;", "Course" }
                                th { style: "padding: 8px;", "Component" }
                                th { style: "padding: 8px; text-align: right;", "Marks" }
                            }
                        }
                        tbody {
                            for item in list {
                                tr {
                                    style: "border-bottom: 1px solid #f1f5f9;",
                                    td { style: "padding: 8px;",
                                        div { style: "font-weight: 600;", "{item.code}" }
                                        div { style: "color: #64748b; font-size: 0.75rem;", "{item.description}" }
                                    }
                                    td { style: "padding: 8px;", "{item.component}" }
                                    td { style: "padding: 8px; text-align: right; font-weight: 600;", "{item.marks} / {item.max_marks}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GradesView(gpa: Option<sync::GpaSummary>, list: Vec<sync::GradeRecord>) -> Element {
    rsx! {
        div {
            h2 { style: "font-size: 1.1rem; margin-top: 0;", "Grades & GPA" }
            if let Some(summary) = gpa {
                div {
                    style: "background: #eff6ff; padding: 12px; border-radius: 6px; margin-bottom: 16px;",
                    if let Some(cgpa) = summary.cgpa {
                        div { style: "font-size: 1.1rem; font-weight: bold; color: #1e40af;", "CGPA: {cgpa}" }
                    }
                    if !summary.sgpa_list.is_empty() {
                        div {
                            style: "display: flex; gap: 12px; margin-top: 8px; flex-wrap: wrap;",
                            for (sem, sgpa) in summary.sgpa_list {
                                span { style: "font-size: 0.85rem; color: #1e3a8a;", "Sem {sem}: {sgpa}" }
                            }
                        }
                    }
                }
            }
            if list.is_empty() {
                p { "No grades recorded." }
            } else {
                div {
                    style: "overflow-x: auto;",
                    table {
                        style: "width: 100%; border-collapse: collapse; font-size: 0.85rem;",
                        thead {
                            tr {
                                style: "background: #f8fafc; text-align: left;",
                                th { style: "padding: 8px;", "Sem" }
                                th { style: "padding: 8px;", "Course" }
                                th { style: "padding: 8px; text-align: center;", "Grade" }
                            }
                        }
                        tbody {
                            for item in list {
                                tr {
                                    style: "border-bottom: 1px solid #f1f5f9;",
                                    td { style: "padding: 8px;", "{item.semester}" }
                                    td { style: "padding: 8px;",
                                        div { style: "font-weight: 600;", "{item.code}" }
                                        div { style: "color: #64748b; font-size: 0.75rem;", "{item.description}" }
                                    }
                                    td { style: "padding: 8px; text-align: center; font-weight: bold;", "{item.grade}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExamsView(list: Vec<sync::ExamScheduleItem>) -> Element {
    rsx! {
        div {
            h2 { style: "font-size: 1.1rem; margin-top: 0;", "Exam Schedule" }
            if list.is_empty() {
                p { "No upcoming exams scheduled." }
            } else {
                div {
                    style: "overflow-x: auto;",
                    table {
                        style: "width: 100%; border-collapse: collapse; font-size: 0.85rem;",
                        thead {
                            tr {
                                style: "background: #f8fafc; text-align: left;",
                                th { style: "padding: 8px;", "Date / Time" }
                                th { style: "padding: 8px;", "Course" }
                                th { style: "padding: 8px;", "Status" }
                            }
                        }
                        tbody {
                            for item in list {
                                tr {
                                    style: "border-bottom: 1px solid #f1f5f9;",
                                    td { style: "padding: 8px;", "{item.date} {item.time}" }
                                    td { style: "padding: 8px;",
                                        div { style: "font-weight: 600;", "{item.code}" }
                                        div { style: "color: #64748b; font-size: 0.75rem;", "{item.description}" }
                                    }
                                    td { style: "padding: 8px;", "{item.status}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HostelView(hostel: Option<sync::HostelDetails>) -> Element {
    if let Some(h) = hostel {
        rsx! {
            div {
                h2 { style: "font-size: 1.1rem; margin-top: 0;", "Hostel Details" }
                InfoRow { label: "Hostel Name", value: h.hostel_name }
                InfoRow { label: "Room No", value: h.room_no }
                InfoRow { label: "Allocation Date", value: h.allocation_date }
                InfoRow { label: "Mess Name", value: h.mess_name }
            }
        }
    } else {
        rsx! { p { "No hostel allocation found." } }
    }
}

#[component]
fn AbsentHoursView(list: Vec<sync::AbsentHourRecord>) -> Element {
    rsx! {
        div {
            h2 { style: "font-size: 1.1rem; margin-top: 0;", "Absent Hours" }
            if list.is_empty() {
                p { "No absent hours recorded." }
            } else {
                div {
                    style: "overflow-x: auto;",
                    table {
                        style: "width: 100%; border-collapse: collapse; font-size: 0.85rem;",
                        thead {
                            tr {
                                style: "background: #f8fafc; text-align: left;",
                                th { style: "padding: 8px;", "Date" }
                                th { style: "padding: 8px;", "Course" }
                                th { style: "padding: 8px; text-align: center;", "Hour" }
                            }
                        }
                        tbody {
                            for item in list {
                                tr {
                                    style: "border-bottom: 1px solid #f1f5f9;",
                                    td { style: "padding: 8px;", "{item.date}" }
                                    td { style: "padding: 8px;",
                                        div { style: "font-weight: 600;", "{item.code}" }
                                        div { style: "color: #64748b; font-size: 0.75rem;", "{item.description}" }
                                    }
                                    td { style: "padding: 8px; text-align: center;", "{item.hour}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CoursesView(list: Vec<sync::CourseHistoryItem>) -> Element {
    rsx! {
        div {
            h2 { style: "font-size: 1.1rem; margin-top: 0;", "Course History" }
            if list.is_empty() {
                p { "No course history found." }
            } else {
                div {
                    style: "overflow-x: auto;",
                    table {
                        style: "width: 100%; border-collapse: collapse; font-size: 0.85rem;",
                        thead {
                            tr {
                                style: "background: #f8fafc; text-align: left;",
                                th { style: "padding: 8px;", "Course" }
                                th { style: "padding: 8px; text-align: center;", "Sem" }
                                th { style: "padding: 8px; text-align: center;", "Credits" }
                            }
                        }
                        tbody {
                            for item in list {
                                tr {
                                    style: "border-bottom: 1px solid #f1f5f9;",
                                    td { style: "padding: 8px;",
                                        div { style: "font-weight: 600;", "{item.code}" }
                                        div { style: "color: #64748b; font-size: 0.75rem;", "{item.description}" }
                                    }
                                    td { style: "padding: 8px; text-align: center;", "{item.semester}" }
                                    td { style: "padding: 8px; text-align: center;", "{item.credits}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
