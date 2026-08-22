use nonprofit_service_metrics::{
    campaign_metrics, infrai_metrics::MetricsClient, DonorReceipt, VolunteerReminder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = DonorReceipt {
        receipt_id: "rcpt-1042".into(),
        campaign: "winter-shelter".into(),
        amount_usd: 125.0,
    };
    let reminders = vec![
        VolunteerReminder {
            reminder_id: "rem-21".into(),
            campaign: "winter-shelter".into(),
            due: true,
            completed: false,
        },
        VolunteerReminder {
            reminder_id: "rem-22".into(),
            campaign: "winter-shelter".into(),
            due: true,
            completed: true,
        },
    ];

    let metrics = campaign_metrics(&receipt, &reminders);
    let client = MetricsClient::from_env()?;
    for metric in &metrics {
        client.report(metric).await?;
    }

    println!(
        "reported receipt={}, amount_usd={}, reminders_due={}",
        receipt.receipt_id, receipt.amount_usd, metrics[2].value
    );
    Ok(())
}

