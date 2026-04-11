use rusqlite::{params, Connection};

pub(crate) struct OrgKnowledge {
    pub(crate) context_hint: String,
    pub(crate) summary: String,
}

pub(crate) fn load_org_knowledge(conn: &Connection, org_id: &str) -> Result<OrgKnowledge, String> {
    let (mission, objectives): (String, String) = conn
        .query_row(
            "SELECT mission, objectives FROM ipc_orgs WHERE id = ?1",
            [org_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "org not found".to_string())?;

    let team: Vec<String> = conn
        .prepare(
            "SELECT agent, role FROM ipc_org_members WHERE org_id = ?1 \
             ORDER BY joined_at DESC LIMIT 10",
        )
        .and_then(|mut stmt| {
            stmt.query_map([org_id], |r| {
                Ok(format!(
                    "{} ({})",
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?
                ))
            })?
            .collect()
        })
        .unwrap_or_default();

    let knowledge = load_knowledge_items(conn, org_id);
    let team_summary = if team.is_empty() {
        "no active members registered".to_string()
    } else {
        team.join(", ")
    };

    let mut summary = format!(
        "Org: {org_id}\nMission: {mission}\nObjectives: {objectives}\nTeam: {team_summary}"
    );
    if !knowledge.is_empty() {
        summary.push_str("\n\nKnowledge Base:");
        for (title, content) in knowledge {
            summary.push_str(&format!("\n\n## {title}\n{content}"));
        }
    }

    Ok(OrgKnowledge {
        context_hint: format!("Org {org_id}: {mission}"),
        summary,
    })
}

pub(crate) fn load_knowledge_items(conn: &Connection, org_id: &str) -> Vec<(String, String)> {
    let prefixed = format!("org:{org_id}");
    conn.prepare(
        "SELECT title, content FROM knowledge_base \
         WHERE domain IN (?1, ?2) ORDER BY title",
    )
    .and_then(|mut stmt| {
        stmt.query_map(params![org_id, prefixed], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect()
    })
    .unwrap_or_default()
}
