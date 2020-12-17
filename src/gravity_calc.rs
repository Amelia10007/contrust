use crate::mass::MassPoint;
use crate::tree::TreeNode;
use crate::type_alias::*;
use apply::Apply;
use dimensioned::Sqrt;
use itertools::Itertools;
use pair_macro::Pair;

/// 静止した質点を表す．
#[derive(Debug, Copy, Clone)]
struct StaticMassPoint {
    mass: Kilogram,
    position: Pair<Meter>,
}

impl StaticMassPoint {
    pub fn from_mass_point(m: MassPoint) -> StaticMassPoint {
        Self {
            mass: m.mass,
            position: m.position,
        }
    }
}

/// 長方形をなす宇宙の部分集合．
#[derive(Debug, Copy, Clone, Default)]
struct Rect {
    /// この領域内にある質量．
    mass: Kilogram,
    /// この領域の重心．
    mass_center: Pair<Meter>,
    /// この長方形領域の中心座標．
    geometric_center: Pair<Meter>,
    /// この長方形領域の大きさ．(中心からの距離ではないことに注意)
    length: Pair<Meter>,
}

impl Rect {
    fn new(
        mass: Kilogram,
        mass_center: Pair<Meter>,
        geometric_center: Pair<Meter>,
        length: Pair<Meter>,
    ) -> Rect {
        Self {
            mass,
            mass_center,
            geometric_center,
            length,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ChildRectLocation {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

impl ChildRectLocation {
    fn locate(pos: Pair<Meter>, geometric_center: Pair<Meter>) -> ChildRectLocation {
        if pos.x < geometric_center.x {
            if pos.y < geometric_center.y {
                ChildRectLocation::LeftTop
            } else {
                ChildRectLocation::LeftBottom
            }
        } else {
            if pos.y < geometric_center.y {
                ChildRectLocation::RightTop
            } else {
                ChildRectLocation::RightBottom
            }
        }
    }

    fn to_geometric_center(self, parent: &Rect) -> Pair<Meter> {
        let shift = match self {
            ChildRectLocation::LeftTop => Pair::new(-0.5, -0.5),
            ChildRectLocation::LeftBottom => Pair::new(-0.5, 0.5),
            ChildRectLocation::RightTop => Pair::new(0.5, -0.5),
            ChildRectLocation::RightBottom => Pair::new(0.5, 0.5),
        };

        parent.geometric_center + parent.length.map_entrywise(shift, |len, shift| len * shift)
    }
}

/// # Params
/// 1. `mass_points` 質点
/// 1. `accels` 計算結果の格納先．
/// 1. `gravity_constant` 万有引力定数．
/// 1. `minimum_ratio_for_integration` `質点-グリッド重心間距離/グリッド長さ`がこの値を上回ったら，グリッド内の情報を統合した引力計算を行う．
/// この値が大きいほど計算は万有引力則に則ったものになるが計算は遅くなり，小さいほど計算が早くなるが正確性は落ちる．
pub fn calculate_accels(
    mass_points: &[MassPoint],
    accels: &mut [Pair<Accel>],
    gravity_constant: GravityConstant,
    minimum_ratio_for_integration: Unitless,
    gravity_cutoff: Meter,
) {
    let root = {
        let mut root = construct_root(mass_points);
        let mass_points = mass_points
            .iter()
            .copied()
            .map(StaticMassPoint::from_mass_point);
        construct_tree(&mut root, mass_points);
        root
    };

    mass_points
        .iter()
        .copied()
        .map(StaticMassPoint::from_mass_point)
        .zip_eq(accels.iter_mut())
        .for_each(|(mass_point, accel)| {
            *accel = calculate_accel(
                mass_point,
                &root,
                gravity_constant,
                gravity_cutoff,
                minimum_ratio_for_integration,
            )
        });
}

fn construct_root(mass_points: &[MassPoint]) -> TreeNode<Rect> {
    let x = mass_points
        .iter()
        .map(|m| m.position.x)
        .minmax_by(|a, b| a.partial_cmp(b).unwrap())
        .into_option();
    let y = mass_points
        .iter()
        .map(|m| m.position.x)
        .minmax_by(|a, b| a.partial_cmp(b).unwrap())
        .into_option();

    let rect = if let (Some((x_min, x_max)), Some((y_min, y_max))) = (x, y) {
        let min = Pair::new(x_min, y_min);
        let max = Pair::new(x_max, y_max);
        let center = (max + min) / 2.0;
        let length = max - min;
        Rect::new(Default::default(), center, center, length)
    } else {
        Rect::default()
    };

    TreeNode::root(rect)
}

fn construct_tree<I: ExactSizeIterator<Item = StaticMassPoint>>(
    rect: &mut TreeNode<Rect>,
    mut mass_points: I,
) {
    match mass_points.len() {
        0 => {}
        1 => {
            let m = mass_points.next().unwrap();
            rect.data_mut().mass = m.mass;
            rect.data_mut().mass_center = m.position;
        }
        _ => {
            let map = mass_points
                .map(|mp| {
                    let location =
                        ChildRectLocation::locate(mp.position, rect.data().geometric_center);
                    (location, mp)
                })
                .into_group_map();

            for (location, mass_points) in map
                .into_iter()
                .filter(|(_, mass_points)| !mass_points.is_empty())
            {
                let child_geometric_center = location.to_geometric_center(rect.data());
                let child_length = rect.data().length / 2.0;
                let mut child_rect = Rect::new(
                    Default::default(),
                    Default::default(),
                    child_geometric_center,
                    child_length,
                )
                .apply(TreeNode::root);
                construct_tree(&mut child_rect, mass_points.into_iter());
                rect.add_child(child_rect);
            }

            let weight_sum = rect
                .children()
                .iter()
                .map(|child| child.data().mass)
                .fold(Default::default(), |acc, cur| acc + cur);

            let mass_center = rect
                .children()
                .iter()
                .map(|child| child.data().mass_center * child.data().mass)
                .fold(Pair::<KilogramMeter>::default(), |acc, cur| acc + cur)
                / weight_sum;

            rect.data_mut().mass = weight_sum;
            rect.data_mut().mass_center = mass_center;
            assert!(weight_sum.value_unsafe > 0.0);
        }
    }
}

fn calculate_accel(
    mass_point: StaticMassPoint,
    rect: &TreeNode<Rect>,
    gravity_constant: GravityConstant,
    gravity_cutoff: Meter,
    minimum_ratio_for_integration: Unitless,
) -> Pair<Accel> {
    let distance_condition = {
        let rect = rect.data();
        let norm = (mass_point.position - rect.mass_center)
            .into_iter()
            .fold(Meter2::new(1.0), |acc, cur| acc + cur * cur);
        let len2 = rect
            .length
            .into_iter()
            .fold(Meter2::new(1.0), |acc, cur| acc + cur * cur);
        let ratio = norm / len2;

        // 質点と重心が十分離れていれば，領域内の全質点の内容を統合しておｋ
        ratio > minimum_ratio_for_integration
    };
    // 領域に子がもうなければ統合しておｋ．なぜならその領域には質点が1つしかないから．
    let tree_condition = rect.is_leaf();

    if distance_condition || tree_condition {
        let rect = rect.data();
        // 重心がこの質点の位置と一致する場合，自身が自身に与える引力を計算することになってしまう．
        // 実際にはそんなことはないので，上記のケースはとばす．
        if rect.mass_center == mass_point.position {
            Default::default()
        } else {
            accel_between(mass_point, rect, gravity_constant, gravity_cutoff)
        }
    } else {
        debug_assert!(!rect.is_leaf());
        rect.children()
            .iter()
            .map(|child| {
                calculate_accel(
                    mass_point,
                    child,
                    gravity_constant,
                    gravity_cutoff,
                    minimum_ratio_for_integration,
                )
            })
            .fold(Default::default(), |acc, cur| acc + cur)
    }
}

fn accel_between(
    receiver: StaticMassPoint,
    applier: &Rect,
    gravity_constant: GravityConstant,
    gravity_cutoff: Meter,
) -> Pair<Accel> {
    let diff = applier.mass_center - receiver.position;
    let square_sum = diff
        .map(|d| d * d)
        .into_iter()
        .fold(Meter2::default(), |acc, cur| acc + cur);

    let normal_diff = diff / square_sum.sqrt();

    let cutoff = gravity_cutoff * gravity_cutoff;
    let len = square_sum + cutoff;

    normal_diff * gravity_constant * applier.mass / len
}
