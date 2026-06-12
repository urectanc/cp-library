/// # Reference
/// - [[Library Checker] Point Set Tree Path Composite Sum](https://maspypy.com/library-checker-point-set-tree-path-composite-sum)
/// - [yosupo-library](https://yosupo06.github.io/yosupo-library/src/yosupo/toptree.hpp)
use algebra::Monoid;
use heavy_light_decomposition::HeavyLightDecomposition;

mod builder;

// TODO: 設計を再検討する
// - 木と載せるデータを分離すべきか？
//   - セグメント木と同様、内部ノードについては公開したくない
//   - 分割統治だけ行う場合には木だけのほうが都合がよい
// - rerootingをどう扱うのがよいか
//   - reverseに集約していくと、結果をPathで返す必要がある
// - prodをupdateくらいきれいに書きたい
pub trait TreeDP {
    type Path: Clone;
    type Point: Clone;
    type Vertex;
    type PathMonoid: Monoid<Elem = Self::Path>;
    type PointMonoid: Monoid<Elem = Self::Point>;
    fn add_edge(path: &Self::Path) -> Self::Point;
    fn add_vertex(point: &Self::Point, vertex: &Self::Vertex) -> Self::Path;
    fn compress(parent: &Self::Path, child: &Self::Path) -> Self::Path {
        Self::PathMonoid::op(parent, child)
    }
    fn rake(lhs: &Self::Point, rhs: &Self::Point) -> Self::Point {
        Self::PointMonoid::op(lhs, rhs)
    }
}

pub struct StaticTopTree<DP: TreeDP> {
    path_tree: Vec<PathNode<DP>>,
    point_tree: Vec<PointNode<DP>>,
    vertex: Vec<VertexNode<DP>>,
    dp: DP::Path,
}

impl<DP: TreeDP> StaticTopTree<DP> {
    pub fn new(hld: &HeavyLightDecomposition, vertex: Vec<DP::Vertex>) -> Self {
        let builder = builder::Builder::<DP>::new(vertex);
        builder.build(hld)
    }

    pub fn update(&mut self, v: usize, vertex: DP::Vertex) {
        self.vertex[v].vertex = vertex;

        let mut path = self.vertex[v].val();
        let mut link = self.vertex[v].link;
        while link != Link::Root {
            while let Link::Path(p) = link {
                let (i, j) = (p as usize >> 1, p as usize & 1);
                self.path_tree[i].child[j] = path;
                path = self.path_tree[i].val();
                link = self.path_tree[i].link;
            }

            let mut point = DP::add_edge(&path);
            while let Link::Point(p) = link {
                let (i, j) = (p as usize >> 1, p as usize & 1);
                self.point_tree[i].child[j] = point;
                point = self.point_tree[i].val();
                link = self.point_tree[i].link;
            }

            if let Link::Vertex(u) = link {
                let u = u as usize;
                self.vertex[u].light_child = point;
                path = self.vertex[u].val();
                link = self.vertex[u].link;
            }
        }

        self.dp = path;
    }

    pub fn dp(&self) -> DP::Point {
        DP::add_edge(&self.dp)
    }

    pub fn prod(&self, mut v: usize) -> DP::Path {
        let mut link = self.vertex[v].link;
        let mut point = self.vertex[v].light_child.clone();
        let mut path = DP::PathMonoid::identity();
        loop {
            let mut above = DP::PathMonoid::identity();
            let mut below = DP::PathMonoid::identity();
            while let Link::Path(p) = link {
                let (i, j) = (p as usize >> 1, p as usize & 1);
                if j == 0 {
                    below = DP::compress(&below, &self.path_tree[i].child[1]);
                } else {
                    above = DP::compress(&self.path_tree[i].child[0], &above);
                }
                link = self.path_tree[i].link;
            }

            point = DP::rake(&point, &DP::add_edge(&below));
            path = DP::compress(
                &above,
                &DP::compress(&DP::add_vertex(&point, &self.vertex[v].vertex), &path),
            );

            if link == Link::Root {
                return path;
            }

            point = DP::PointMonoid::identity();
            while let Link::Point(p) = link {
                let (i, j) = (p as usize >> 1, p as usize & 1);
                point = if j == 0 {
                    DP::rake(&point, &self.point_tree[i].child[1])
                } else {
                    DP::rake(&self.point_tree[i].child[0], &point)
                };
                link = self.point_tree[i].link;
            }

            if let Link::Vertex(u) = link {
                v = u as usize;
                link = self.vertex[v].link;
            }
        }
    }
}

struct PathNode<DP: TreeDP> {
    link: Link,
    child: [DP::Path; 2],
}

impl<DP: TreeDP> PathNode<DP> {
    fn new(left: DP::Path, right: DP::Path) -> Self {
        Self {
            link: Link::Root,
            child: [left, right],
        }
    }

    fn val(&self) -> DP::Path {
        DP::compress(&self.child[0], &self.child[1])
    }
}

struct PointNode<DP: TreeDP> {
    link: Link,
    child: [DP::Point; 2],
}

impl<DP: TreeDP> PointNode<DP> {
    fn new(left: DP::Point, right: DP::Point) -> Self {
        Self {
            link: Link::Root,
            child: [left, right],
        }
    }

    fn val(&self) -> DP::Point {
        DP::rake(&self.child[0], &self.child[1])
    }
}

struct VertexNode<DP: TreeDP> {
    link: Link,
    vertex: DP::Vertex,
    light_child: DP::Point,
}

impl<DP: TreeDP> VertexNode<DP> {
    fn new(vertex: DP::Vertex) -> Self {
        Self {
            link: Link::Root,
            vertex,
            light_child: DP::PointMonoid::identity(),
        }
    }

    fn val(&self) -> DP::Path {
        DP::add_vertex(&self.light_child, &self.vertex)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Link {
    Path(u32),
    Point(u32),
    Vertex(u32),
    Root,
}
