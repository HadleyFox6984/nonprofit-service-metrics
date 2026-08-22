pub mod infrai_metrics;

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct DonorReceipt {
    pub receipt_id: String,
    pub campaign: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone)]
pub struct VolunteerReminder {
    pub reminder_id: String,
    pub campaign: String,
    pub due: bool,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    #[serde(rename = "type")]
    pub metric_type: String,
    pub tags: serde_json::Value,
    pub idempotency_key: String,
}

/// Turn one service snapshot into stable metric writes.
pub fn campaign_metrics(
    receipt: &DonorReceipt,
    reminders: &[VolunteerReminder],
) -> Vec<MetricPoint> {
    let due_count = reminders
        .iter()
        .filter(|item| item.campaign == receipt.campaign && item.due && !item.completed)
        .count() as f64;

    vec![
        MetricPoint {
            name: "nonprofit.donor_receipts".into(),
            value: 1.0,
            metric_type: "counter".into(),
            tags: serde_json::json!({ "campaign": receipt.campaign }),
            idempotency_key: format!("receipt-count-{}", receipt.receipt_id),
        },
        MetricPoint {
            name: "nonprofit.donations_usd".into(),
            value: receipt.amount_usd,
            metric_type: "counter".into(),
            tags: serde_json::json!({ "campaign": receipt.campaign }),
            idempotency_key: format!("receipt-value-{}", receipt.receipt_id),
        },
        MetricPoint {
            name: "nonprofit.volunteer_reminders_due".into(),
            value: due_count,
            metric_type: "gauge".into(),
            tags: serde_json::json!({ "campaign": receipt.campaign }),
            idempotency_key: format!("reminders-due-{}", receipt.receipt_id),
        },
    ]
}

