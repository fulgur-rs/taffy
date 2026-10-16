use super::*;

#[test]
fn restoring_a_speculative_child_restores_float_slots_clearance_and_strut_state() {
    let mut bfc = BlockFormattingContext::new();
    let mut ctx = bfc.root_block_context();
    ctx.set_width(200.0);
    ctx.place_floated_box(Size { width: 20.0, height: 30.0 }, 10.0, FloatDirection::Left, Clear::None, true);
    let saved = ctx.snapshot();
    {
        let mut child = ctx.sub_context(40.0, [7.0, 9.0]);
        child.set_width(120.0);
        child.place_floated_box(Size { width: 60.0, height: 80.0 }, 0.0, FloatDirection::Right, Clear::None, true);
    }
    ctx.add_child_floated_content_height_contribution(120.0);
    ctx.apply_content_box_inset([30.0, 30.0]);
    ctx.commit_strut();
    ctx.restore(&saved);

    assert_eq!(ctx.cleared_threshold(Clear::Both), Some(40.0));
    assert_eq!(ctx.floated_content_height_contribution(), 40.0);
    assert!(ctx.has_adjoining_float(Clear::Left));
    assert!(!ctx.has_adjoining_float(Clear::Right));
    assert_eq!(ctx.top_adjoining_floats(), [true, false]);
    let slot = ctx.find_content_slot(10.0, Clear::None, None);
    assert_eq!((slot.x, slot.y, slot.width), (20.0, 10.0, 180.0));
    assert_eq!(ctx.bfc.float_context.left_floats().len(), 1);
    assert_eq!(ctx.bfc.float_context.right_floats().len(), 0);
    let placed =
        ctx.place_floated_box(Size { width: 30.0, height: 20.0 }, 10.0, FloatDirection::Left, Clear::None, false);
    assert_eq!(placed, Point { x: 20.0, y: 10.0 });
}
