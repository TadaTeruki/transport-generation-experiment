use rstar::{RTree, RTreeObject, AABB};

use crate::Site2D;

pub(crate) enum PathTreeQuery<'a> {
    None,
    Site(usize),
    Path(&'a PathTreeObject),
}

pub(crate) struct PathTreeObject {
    pub path_index: usize,
    pub site_index_start: usize,
    pub site_index_end: usize,
    pub site_start: Site2D,
    pub site_end: Site2D,
}

impl RTreeObject for PathTreeObject {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [
                self.site_start.x.min(self.site_end.x),
                self.site_start.y.min(self.site_end.y),
            ],
            [
                self.site_start.x.max(self.site_end.x),
                self.site_start.y.max(self.site_end.y),
            ],
        )
    }
}

impl PartialEq for PathTreeObject {
    fn eq(&self, other: &Self) -> bool {
        self.path_index == other.path_index
    }
}

impl Eq for PathTreeObject {}

pub(crate) struct PathTree {
    tree: RTree<PathTreeObject>,
    next_path_index: usize,
}

impl PathTree {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
            next_path_index: 0,
        }
    }

    pub fn insert(
        &mut self,
        site_index_start: usize,
        site_index_end: usize,
        site_start: Site2D,
        site_end: Site2D,
    ) {
        let path_index = self.next_path_index;
        self.next_path_index += 1;
        self.tree.insert(PathTreeObject {
            path_index: path_index,
            site_start,
            site_end,
            site_index_start,
            site_index_end,
        });
    }

    pub fn find(
        &self,
        site_start: &Site2D,
        site_end: &Site2D,
        diameter: f64,
        indices_not_including: &[usize],
    ) -> PathTreeQuery {
        let envelope = AABB::from_corners(
            [site_end.x - diameter, site_end.y - diameter],
            [site_end.x + diameter, site_end.y + diameter],
        );
        let result = self.tree.locate_in_envelope_intersecting(&envelope);
        let mut min_distance = diameter;
        let mut min_path = None;
        for item in result {
            if indices_not_including.contains(&item.site_index_end)
                || indices_not_including.contains(&item.site_index_start)
            {
                continue;
            }
            let distance_line = ((item.site_end.y - item.site_start.y) * site_start.x
                - (item.site_end.x - item.site_start.x) * site_start.y
                + item.site_end.x * item.site_start.y
                - item.site_end.y * item.site_start.x)
                .abs()
                / ((item.site_end.y - item.site_start.y).powi(2)
                    + (item.site_end.x - item.site_start.x).powi(2))
                .sqrt();

            if distance_line < min_distance {
                min_distance = distance_line;
                min_path = Some(item);
            }
        }

        if let Some(min_path) = min_path {
            let squared_distance_item_start = (site_end.x - min_path.site_start.x).powi(2)
                + (site_end.y - min_path.site_start.y).powi(2);
            if squared_distance_item_start < diameter.powi(2) {
                return PathTreeQuery::Site(min_path.site_index_start);
            }
            let squared_distance_item_end = (site_end.x - min_path.site_end.x).powi(2)
                + (site_end.y - min_path.site_end.y).powi(2);
            if squared_distance_item_end < diameter.powi(2) {
                return PathTreeQuery::Site(min_path.site_index_end);
            }

            return PathTreeQuery::Path(min_path);
        }

        PathTreeQuery::None
    }

    pub fn remove(&mut self, path_object: &PathTreeObject) {
        self.tree.remove(path_object);
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&PathTreeObject),
    {
        self.tree.iter().for_each(|item| f(item));
    }
}
