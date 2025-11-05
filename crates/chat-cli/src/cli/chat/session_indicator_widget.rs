use std::io::Write;
use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use eyre::Result;

use super::coordinator::MultiSessionCoordinator;

pub struct SessionIndicator;

impl SessionIndicator {
    pub fn render<W: Write>(writer: &mut W, coordinator: &MultiSessionCoordinator) -> Result<()> {
        // Get session info
        let (all_sessions, waiting_sessions) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let all = coordinator.list_sessions().await;
                let waiting = coordinator.get_waiting_sessions().await;
                (all, waiting)
            })
        });

        if all_sessions.is_empty() {
            return Ok(());
        }

        // Just print inline before the prompt - simpler and works with scrolling
        writeln!(writer)?;
        execute!(writer, SetForegroundColor(Color::DarkGrey))?;
        write!(writer, "╔═ Sessions: ")?;
        
        for (idx, name) in all_sessions.iter().take(3).enumerate() {
            if idx > 0 {
                write!(writer, ", ")?;
            }
            
            let icon_color = if waiting_sessions.contains(name) {
                Color::Yellow
            } else {
                Color::Green
            };
            
            execute!(writer, SetForegroundColor(icon_color))?;
            write!(writer, "{}", name)?;
            execute!(writer, SetForegroundColor(Color::DarkGrey))?;
        }
        
        if all_sessions.len() > 3 {
            write!(writer, " +{}", all_sessions.len() - 3)?;
        }
        
        writeln!(writer, " ═╗")?;
        execute!(writer, ResetColor)?;
        
        writer.flush()?;

        Ok(())
    }
}
