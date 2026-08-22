use nonprofit_service_metrics::{campaign_metrics, DonorReceipt, VolunteerReminder};

#[test]
fn reports_only_due_unfinished_reminders_for_the_receipt_campaign() {
    let receipt = DonorReceipt {
        receipt_id: "rcpt-7".into(),
        campaign: "food-drive".into(),
        amount_usd: 80.0,
    };
    let reminders = vec![
        reminder("food-drive", true, false),
        reminder("food-drive", true, true),
        reminder("food-drive", false, false),
        reminder("clinic", true, false),
    ];

    let points = campaign_metrics(&receipt, &reminders);

    assert_eq!(points[0].value, 1.0);
    assert_eq!(points[1].value, 80.0);
    assert_eq!(points[2].value, 1.0);
    assert_eq!(points[2].metric_type, "gauge");
    assert_eq!(points[0].idempotency_key, "receipt-count-rcpt-7");
}

fn reminder(campaign: &str, due: bool, completed: bool) -> VolunteerReminder {
    VolunteerReminder {
        reminder_id: format!("{campaign}-{due}-{completed}"),
        campaign: campaign.into(),
        due,
        completed,
    }
}

