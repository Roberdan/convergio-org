//! Domain-aware department generation.

use super::{agent, dept_with_agents, Department};

/// Detect domains from readme + mission, return extra departments.
pub fn domain_departments(readme: &str, mission: &str, slug: &str) -> Vec<Department> {
    let text = format!("{readme} {mission}").to_lowercase();
    let mut d = Vec::new();
    // Healthcare / rehabilitation / disability
    if kw_n(&text, MEDICAL, 2) {
        d.push(dept_with_agents(
            "Medical & Rehab",
            vec![
                ag(
                    slug,
                    "clinical-advisor",
                    S,
                    "clinical protocol review",
                    "Clinical Advisor",
                    &["clinical protocols", "evidence-based practice"],
                ),
                ag(
                    slug,
                    "accessibility-lead",
                    S,
                    "WCAG compliance, assistive tech",
                    "Accessibility Specialist",
                    &["WCAG", "motor accessibility", "adaptive interfaces"],
                ),
            ],
        ));
        d.push(dept_with_agents(
            "Psychology & Education",
            vec![
                ag(
                    slug,
                    "child-psychologist",
                    S,
                    "developmental psychology, gamification",
                    "Child Psychologist",
                    &["developmental psychology", "gamification", "engagement"],
                ),
                ag(
                    slug,
                    "educator",
                    H,
                    "special education, learning outcomes",
                    "Educational Specialist",
                    &[
                        "special education",
                        "curriculum design",
                        "learning assessment",
                    ],
                ),
            ],
        ));
    }
    // Enterprise / business process
    if kw(&text, ENTERPRISE) {
        d.push(dept_with_agents(
            "Business Process",
            vec![
                ag(
                    slug,
                    "process-architect",
                    S,
                    "workflow optimization, enterprise patterns",
                    "Process Architect",
                    &["BPMN", "workflow optimization", "enterprise architecture"],
                ),
                ag(
                    slug,
                    "change-mgmt",
                    H,
                    "change management, stakeholder comms",
                    "Change Manager",
                    &[
                        "change management",
                        "adoption tracking",
                        "stakeholder comms",
                    ],
                ),
            ],
        ));
    }
    // GDPR / compliance / privacy
    if kw(&text, COMPLIANCE) {
        d.push(dept_with_agents(
            "Compliance & Legal",
            vec![
                ag(
                    slug,
                    "compliance-officer",
                    S,
                    "regulatory compliance, GDPR",
                    "Compliance Officer",
                    &["GDPR", "data governance", "regulatory compliance"],
                ),
                ag(
                    slug,
                    "privacy-analyst",
                    H,
                    "privacy impact, consent, PII",
                    "Privacy Analyst",
                    &["privacy-by-design", "PII detection", "consent management"],
                ),
            ],
        ));
    }
    // Backup / data protection / security
    if kw(&text, BACKUP) {
        d.push(dept_with_agents(
            "Data Protection",
            vec![
                ag(
                    slug,
                    "security-engineer",
                    S,
                    "encryption, access control, integrity",
                    "Security Engineer",
                    &["encryption", "access control", "threat modeling"],
                ),
                ag(
                    slug,
                    "disaster-recovery",
                    H,
                    "DR planning, backup validation",
                    "DR Specialist",
                    &["disaster recovery", "backup validation", "RPO/RTO"],
                ),
            ],
        ));
    }
    // Design system / UI framework
    if kw(&text, DESIGN) {
        d.push(dept_with_agents(
            "Design & UX",
            vec![
                ag(
                    slug,
                    "design-system-lead",
                    S,
                    "design tokens, visual consistency",
                    "Design System Lead",
                    &["design tokens", "component API", "visual consistency"],
                ),
                ag(
                    slug,
                    "ux-researcher",
                    H,
                    "usability testing, interaction patterns",
                    "UX Researcher",
                    &["usability testing", "user research", "interaction design"],
                ),
            ],
        ));
    }
    // macOS / native platform
    if kw(&text, MACOS) {
        d.push(dept_with_agents(
            "Platform",
            vec![ag(
                slug,
                "platform-engineer",
                S,
                "macOS APIs, sandboxing, notarization",
                "Platform Engineer",
                &["macOS", "notarization", "sandboxing", "App Store"],
            )],
        ));
    }
    // Microsoft / enterprise integration
    if kw(&text, MSFT) {
        d.push(dept_with_agents(
            "Enterprise Integration",
            vec![ag(
                slug,
                "integration-architect",
                S,
                "Microsoft Graph, Azure AD, SSO",
                "Integration Architect",
                &["Microsoft Graph", "Azure AD", "SSO", "enterprise APIs"],
            )],
        ));
    }
    // AI / LLM safety
    if kw(&text, AI) {
        d.push(dept_with_agents(
            "AI & Ethics",
            vec![ag(
                slug,
                "ai-ethics",
                S,
                "AI safety, bias, responsible AI",
                "AI Ethics Lead",
                &[
                    "responsible AI",
                    "bias detection",
                    "guardrails",
                    "AI safety",
                ],
            )],
        ));
    }
    // Non-profit / social impact
    if kw(&text, IMPACT) {
        d.push(dept_with_agents(
            "Impact & Outreach",
            vec![ag(
                slug,
                "impact-analyst",
                H,
                "social impact, outcome tracking",
                "Impact Analyst",
                &["impact measurement", "outcome tracking", "grant reporting"],
            )],
        ));
    }
    d
}

const S: &str = super::MODEL_OPUS;
const H: &str = super::MODEL_OPUS;
fn ag(slug: &str, suf: &str, m: &str, cap: &str, role: &str, sk: &[&str]) -> super::AgentSpec {
    agent(slug, suf, m, cap, role, sk)
}
fn kw(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|k| text.contains(k))
}
/// Require at least `n` keyword matches (reduces false positives).
fn kw_n(text: &str, keywords: &[&str], n: usize) -> bool {
    keywords.iter().filter(|k| text.contains(*k)).count() >= n
}

use super::keywords::*;
