use rstar::{RTree, RTreeObject, AABB};

use crate::Site2D;

use super::transport::PathAttr;

pub(crate) enum PathTreeQuery<'a> {
    None,
    Site(usize),
    Path(&'a PathTreeObject),
}

#[derive(Clone, Copy)]
pub(crate) struct PathTreeObject {
    pub path_index: usize,
    pub site_index_start: usize,
    pub site_index_end: usize,
    pub site_start: Site2D,
    pub site_end: Site2D,
    pub path_attr: PathAttr,
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
        path_attr: PathAttr,
    ) {
        let path_index = self.next_path_index;
        self.next_path_index += 1;
        self.tree.insert(PathTreeObject {
            path_index,
            site_start,
            site_end,
            site_index_start,
            site_index_end,
            path_attr,
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
        let site_cmp = Site2D {
            x: (site_start.x + site_end.x) * 0.5,
            y: (site_start.y + site_end.y) * 0.5,
        };
        let mut min_distance = diameter;
        let mut min_path = None;
        for item in result {
            if indices_not_including.contains(&item.site_index_end)
                || indices_not_including.contains(&item.site_index_start)
            {
                continue;
            }

            let distance_line = ((item.site_end.y - item.site_start.y) * site_cmp.x
                - (item.site_end.x - item.site_start.x) * site_cmp.y
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
    pub fn split(
        &mut self,
        path_object: PathTreeObject,
        split_site: &Site2D,
        split_site_index: usize,
    ) {
        let remove = self.tree.remove(&path_object);
        if remove.is_none() {
            panic!("aaa");
        }
        self.insert(
            path_object.site_index_start,
            split_site_index,
            path_object.site_start,
            *split_site,
            path_object.path_attr,
        );
        self.insert(
            split_site_index,
            path_object.site_index_end,
            *split_site,
            path_object.site_end,
            path_object.path_attr,
        );
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&PathTreeObject),
    {
        self.tree.iter().for_each(|item| f(item));
    }
}
