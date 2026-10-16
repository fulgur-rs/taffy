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

#[test]
#[cfg(feature = "taffy_tree")]
fn clearance_with_a_negative_leading_margin_does_not_cycle_between_origins() {
    use crate::prelude::*;
    let mut tree: TaffyTree<()> = TaffyTree::new();
    tree.disable_rounding();
    let float = tree
        .new_leaf(Style {
            display: Display::Block,
            float: Float::Left,
            size: Size { width: length(40.0), height: length(90.0) },
            ..Default::default()
        })
        .unwrap();
    let child = tree
        .new_leaf(Style {
            display: Display::Block,
            clear: Clear::Left,
            margin: Rect { top: length(-80.0), ..zero() },
            size: Size { width: auto(), height: length(10.0) },
            ..Default::default()
        })
        .unwrap();
    let parent = tree
        .new_with_children(
            Style { display: Display::Block, margin: Rect { top: length(100.0), ..zero() }, ..Default::default() },
            &[child],
        )
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Block,
                size: Size { width: length(200.0), height: auto() },
                ..Default::default()
            },
            &[float, parent],
        )
        .unwrap();
    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(tree.layout(parent).unwrap().location.y, 100.0);
    assert_eq!(tree.layout(child).unwrap().location.y, -10.0);
}

#[test]
#[cfg(feature = "taffy_tree")]
fn changing_a_context_laid_out_child_invalidates_its_cached_parent() {
    use crate::prelude::*;
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let mut child_style =
        Style { display: Display::Block, size: Size { width: auto(), height: length(10.0) }, ..Default::default() };
    let child = tree.new_leaf(child_style.clone()).unwrap();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Block,
                size: Size { width: length(200.0), height: auto() },
                ..Default::default()
            },
            &[child],
        )
        .unwrap();
    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert!(!tree.dirty(child).unwrap());
    child_style.size.height = length(20.0);
    tree.set_style(child, child_style).unwrap();
    assert!(tree.dirty(root).unwrap());
    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
    assert_eq!(tree.layout(root).unwrap().size.height, 20.0);
    assert_eq!(tree.layout(child).unwrap().size.height, 20.0);
}

#[test]
#[cfg(feature = "taffy_tree")]
fn clearance_stays_finite_across_signed_ancestor_margin_sets() {
    use crate::prelude::*;
    for float_height in [10.0, 90.0] {
        for parent_top in [-100.0, -80.0, -20.0, 0.0, 20.0, 80.0, 100.0] {
            for child_top in [-100.0, -80.0, -20.0, 0.0, 20.0, 80.0, 100.0] {
                for parent_clear in [Clear::None, Clear::Left] {
                    let mut tree: TaffyTree<()> = TaffyTree::new();
                    tree.disable_rounding();
                    let float = tree
                        .new_leaf(Style {
                            display: Display::Block,
                            float: Float::Left,
                            size: Size { width: length(40.0), height: length(float_height) },
                            ..Default::default()
                        })
                        .unwrap();
                    let child = tree
                        .new_leaf(Style {
                            display: Display::Block,
                            clear: Clear::Left,
                            margin: Rect { top: length(child_top), ..zero() },
                            size: Size { width: auto(), height: length(10.0) },
                            ..Default::default()
                        })
                        .unwrap();
                    let parent = tree
                        .new_with_children(
                            Style {
                                display: Display::Block,
                                clear: parent_clear,
                                margin: Rect { top: length(parent_top), ..zero() },
                                ..Default::default()
                            },
                            &[child],
                        )
                        .unwrap();
                    let root = tree
                        .new_with_children(
                            Style {
                                display: Display::Block,
                                size: Size { width: length(200.0), height: auto() },
                                ..Default::default()
                            },
                            &[float, parent],
                        )
                        .unwrap();
                    tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
                    let parent_layout = tree.layout(parent).unwrap();
                    let child_layout = tree.layout(child).unwrap();
                    let child_top = parent_layout.location.y + child_layout.location.y;
                    assert!(child_top.is_finite());
                    assert!(child_top >= float_height);
                }
            }
        }
    }
}
