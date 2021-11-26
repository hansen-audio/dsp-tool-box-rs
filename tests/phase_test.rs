use dsp_tool_box_rs::modulation::phase::*;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_advance() {
        let context = Phase::new();
        let mut value = 0.1;
        let did_overflow = context.advance(&mut value, 1);
        assert_eq!(did_overflow, true);
    }

    #[test]
    fn test_project_synced_overflow() {
        let mut phase_value = 0.;
        let mut context = Phase::new();
        context.set_sync_mode(SyncMode::ProjectSync);
        context.set_note_len(1.0);

        context.set_project_time(3.9);
        let overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, false);

        context.set_project_time(4.0);
        let overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, true);
    }

    #[test]
    fn test_project_synced_overflow_huge_project_time() {
        let mut phase_value = 0.;
        let mut context = Phase::new();
        context.set_sync_mode(SyncMode::ProjectSync);
        context.set_note_len(1.0);

        context.set_project_time(999999.9);
        let overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, false);

        context.set_project_time(4.0);
        let overflow = context.advance(&mut phase_value, 1);
        assert_eq!(overflow, true);
    }

    #[test]
    #[ignore]
    fn test_debug_print() {
        let p = Phase::new();
        println!("{:#?}", p);
    }
}
