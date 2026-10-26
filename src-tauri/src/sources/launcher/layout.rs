use crate::core::settings::WindowLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// The visible frame a window should occupy inside the monitor work area; `None` means leave it alone
/// (or, for `Maximize`, let Windows maximize it).
pub fn target_rect(work: Rect, layout: WindowLayout) -> Option<Rect> {
    let half_width = work.width / 2;
    let half_height = work.height / 2;
    let right_x = work.x + half_width;
    let bottom_y = work.y + half_height;
    let (x, y, width, height) = match layout {
        WindowLayout::None | WindowLayout::Maximize => return None,
        WindowLayout::Left => (work.x, work.y, half_width, work.height),
        WindowLayout::Right => (right_x, work.y, work.width - half_width, work.height),
        WindowLayout::TopLeft => (work.x, work.y, half_width, half_height),
        WindowLayout::TopRight => (right_x, work.y, work.width - half_width, half_height),
        WindowLayout::BottomLeft => (work.x, bottom_y, half_width, work.height - half_height),
        WindowLayout::BottomRight => (right_x, bottom_y, work.width - half_width, work.height - half_height),
    };
    Some(Rect { x, y, width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Rect = Rect { x: 0, y: 0, width: 1921, height: 1032 };

    #[test]
    fn halves_cover_the_whole_work_area_without_gaps() {
        let left = target_rect(WORK, WindowLayout::Left).unwrap();
        let right = target_rect(WORK, WindowLayout::Right).unwrap();
        assert_eq!(left, Rect { x: 0, y: 0, width: 960, height: 1032 });
        assert_eq!(right.x, left.width);
        assert_eq!(left.width + right.width, WORK.width);
    }

    #[test]
    fn quarters_sit_in_their_corners() {
        let bottom_right = target_rect(WORK, WindowLayout::BottomRight).unwrap();
        assert_eq!(bottom_right, Rect { x: 960, y: 516, width: 961, height: 516 });
    }

    #[test]
    fn maximize_and_none_are_handled_by_the_window_itself() {
        assert_eq!(target_rect(WORK, WindowLayout::Maximize), None);
        assert_eq!(target_rect(WORK, WindowLayout::None), None);
    }
}
