//! Linux Bubblewrap prerequisite prompts for `ChatWidget`.

use super::*;
use codex_sandboxing::LinuxSandboxPrerequisiteIssue;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

impl ChatWidget {
    pub(crate) fn open_linux_sandbox_issue_prompt(
        &mut self,
        issue: LinuxSandboxPrerequisiteIssue,
        remaining: Vec<LinuxSandboxPrerequisiteIssue>,
    ) {
        let Some((title, description)) = linux_sandbox_issue_text(&issue) else {
            self.add_warning_message(
                "Bubblewrap failed its self-test, but offcodex could not identify a safe automatic repair. Check that bwrap, pkexec, and sysctl are installed; then inspect container, LSM, and kernel policies. WSL1 requires an upgrade to WSL2.".to_string(),
            );
            if let Some((next, rest)) = remaining.split_first() {
                self.app_event_tx
                    .send(AppEvent::OpenLinuxSandboxIssuePrompt {
                        issue: next.clone(),
                        remaining: rest.to_vec(),
                    });
            }
            return;
        };

        let apply_remaining = remaining.clone();
        let skip_remaining = remaining;
        let mut items = vec![SelectionItem {
            name: "Apply this suggested fix".to_string(),
            description: Some(description.to_string()),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenLinuxSandboxPersistencePrompt {
                    issue: issue.clone(),
                    remaining: apply_remaining.clone(),
                });
            })],
            dismiss_on_select: true,
            ..Default::default()
        }];
        items.push(SelectionItem {
            name: "Skip this fix".to_string(),
            description: Some("Continue without changing this host setting.".to_string()),
            actions: vec![Box::new(move |tx| {
                if let Some((next, rest)) = skip_remaining.split_first() {
                    tx.send(AppEvent::OpenLinuxSandboxIssuePrompt {
                        issue: next.clone(),
                        remaining: rest.to_vec(),
                    });
                }
            })],
            dismiss_on_select: true,
            ..Default::default()
        });

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(title.to_string()),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(Paragraph::new(
                "Bubblewrap cannot create its sandbox. This check found a possible host-side cause. No setting changes unless you explicitly approve the next step.",
            ).wrap(Wrap { trim: false })),
            ..Default::default()
        });
    }

    pub(crate) fn open_linux_sandbox_persistence_prompt(
        &mut self,
        issue: LinuxSandboxPrerequisiteIssue,
        remaining: Vec<LinuxSandboxPrerequisiteIssue>,
    ) {
        let temporary_issue = issue.clone();
        let permanent_issue = issue.clone();
        let temporary_remaining = remaining.clone();
        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some("Choose the scope of this sandbox fix".to_string()),
            footer_hint: Some(standard_popup_hint_line()),
            items: vec![
                SelectionItem {
                    name: "Apply until the next reboot".to_string(),
                    description: Some("Changes a global kernel setting now; it resets after reboot.".to_string()),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::ApplyLinuxSandboxPrerequisite {
                            issue: temporary_issue.clone(),
                            persistent: false,
                            remaining: temporary_remaining.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                },
                SelectionItem {
                    name: "Apply permanently".to_string(),
                    description: Some("Creates a dedicated /etc/sysctl.d/offcodex file and applies it now.".to_string()),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::ApplyLinuxSandboxPrerequisite {
                            issue: permanent_issue.clone(),
                            persistent: true,
                            remaining: remaining.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                },
            ],
            header: Box::new(Paragraph::new(
                "This operation requires PolicyKit authentication. A temporary choice lasts only until the next reboot; Linux cannot scope these kernel settings to one offcodex session.",
            ).wrap(Wrap { trim: false })),
            ..Default::default()
        });
    }
}

fn linux_sandbox_issue_text(
    issue: &LinuxSandboxPrerequisiteIssue,
) -> Option<(&'static str, &'static str)> {
    match issue {
        LinuxSandboxPrerequisiteIssue::EnableUnprivilegedUserNamespaces => Some((
            "Enable unprivileged user namespaces",
            "Set kernel.unprivileged_userns_clone=1 so Bubblewrap can create its user namespace.",
        )),
        LinuxSandboxPrerequisiteIssue::IncreaseUserNamespaceLimit => Some((
            "Increase the user-namespace limit",
            "Set user.max_user_namespaces=1024; the current limit is zero.",
        )),
        LinuxSandboxPrerequisiteIssue::RelaxAppArmorUserNamespaceRestriction => Some((
            "Relax AppArmor's user-namespace restriction",
            "Set kernel.apparmor_restrict_unprivileged_userns=0. This is relevant on Ubuntu systems where AppArmor blocks Bubblewrap's uid mapping.",
        )),
        LinuxSandboxPrerequisiteIssue::ManualInvestigationRequired => None,
    }
}
