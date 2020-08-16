use std::process::exit;

use num_rational::*;


type Pos = (isize, isize);

// TODO add a userdata to the closures
pub fn compute_fov<F, G>(origin: Pos, is_blocking: &mut F, mark_visible: &mut G)
    where F: FnMut(Pos) -> bool,
          G: FnMut(Pos), {

    mark_visible(origin);

    for i in 0..4 {
        let quadrant = Quadrant::new(Cardinal::from_index(i), origin);

        let first_row = Row::new(1, Rational::new(-1, 1), Rational::new(1, 1));

        scan(first_row, quadrant, is_blocking, mark_visible);
    }
}

fn scan<F, G>(row: Row, quadrant: Quadrant, is_blocking: &mut F, mark_visible: &mut G) 
    where F: FnMut(Pos) -> bool,
          G: FnMut(Pos), {
    let mut prev_tile = None;

    let mut row = row;

    for tile in row.tiles() {
        let tile_is_wall = is_blocking(quadrant.transform(tile));
        let tile_is_floor = !tile_is_wall;

        let prev_is_wall = prev_tile.map_or(false, |prev| is_blocking(quadrant.transform(prev)));
        let prev_is_floor = prev_tile.map_or(false, |prev| !is_blocking(quadrant.transform(prev)));

        if tile_is_wall || is_symmetric(row, tile) {
            let pos = quadrant.transform(tile);

            mark_visible(pos);
        }

        if prev_is_wall && tile_is_floor {
            row.start_slope = slope(tile);
        }

        if prev_is_floor && tile_is_wall {
            let mut next_row = row.next();

            next_row.end_slope = slope(tile);

            scan(next_row, quadrant, is_blocking, mark_visible);
        }

        prev_tile = Some(tile);
    }
        
    if prev_tile.map_or(false, |tile| !is_blocking(quadrant.transform(tile))) {
        scan(row.next(), quadrant, is_blocking, mark_visible);
    }
}



#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Cardinal {
    North = 0,
    East  = 1,
    South = 2,
    West  = 3,
}

impl Cardinal {
    pub fn from_index(index: usize) -> Cardinal {
        use Cardinal::*;
        return [North, East, South, West][index];
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Quadrant {
    cardinal: Cardinal,
    ox: isize,
    oy: isize,
}

impl Quadrant {
    pub fn new(cardinal: Cardinal, origin: Pos) -> Quadrant {
        return Quadrant { cardinal, ox: origin.0, oy: origin.1 };
    }

    pub fn transform(&self, tile: Pos) -> Pos{
        let (row, col) = tile;

        match self.cardinal {
            Cardinal::North => {
                return (self.ox + col, self.oy - row);
            }

            Cardinal::South => {
                return (self.ox + col, self.oy + row);
            }

            Cardinal::East => {
                return (self.ox + row, self.oy + col);
            }

            Cardinal::West => {
                return (self.ox - row, self.oy + col);
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Row {
    depth: isize,
    start_slope: Rational,
    end_slope: Rational,
}

impl Row {
    pub fn new(depth: isize, start_slope: Rational, end_slope: Rational) -> Row {
        return Row { depth, start_slope, end_slope, };
    }

    pub fn tiles(&self) -> impl Iterator<Item=Pos> {
        let depth_times_start = Rational::new(self.depth, 1) * self.start_slope;
        let depth_times_end = Rational::new(self.depth, 1) * self.end_slope;

        let min_col = depth_times_start.ceil().to_integer();

        let max_col = depth_times_end.floor().to_integer();

        let depth = self.depth;

        return (min_col..=max_col).map(move |col| (depth, col));
    }

    pub fn next(&self) -> Row {
        return Row::new(self.depth + 1, self.start_slope, self.end_slope);
    }
}

pub fn slope(tile: Pos) -> Rational {
    let (row_depth, col) = tile;
    return Rational::new(2 * col - 1, 2 * row_depth);
}

pub fn is_symmetric(row: Row, tile: Pos) -> bool {
    let (row_depth, col) = tile;

    let depth_times_start = Rational::new(row.depth, 1) * row.start_slope;
    let depth_times_end = Rational::new(row.depth, 1) * row.end_slope;

    let col_rat = Rational::new(col, 1);

    let symmetric = col_rat >= depth_times_start && col_rat <= depth_times_end;
    if tile == (6, 0) {
        dbg!(symmetric);
    }

    return symmetric;
}

pub fn round_ties_up(n: Rational) -> isize {
    return (n + Rational::new(1, 2)).to_integer();
}

pub fn round_ties_down(n: Rational) -> isize {
    return (n - Rational::new(1, 2)).to_integer();
}

#[test]
pub fn test_expansive_walls() {
    let origin = (1, 2);

    let tiles = vec!(vec!(1, 1, 1, 1, 1, 1, 1),
                     vec!(1, 0, 0, 0, 0, 0, 1),
                     vec!(1, 0, 0, 0, 0, 0, 1),
                     vec!(1, 1, 1, 1, 1, 1, 1));

    let mut is_blocking = |pos: Pos| {
        let outside = (pos.1 as usize) >= tiles.len() || (pos.0 as usize) >= tiles[0].len();
        return  outside || tiles[pos.1 as usize][pos.0 as usize] == 1;
    };

    let mut visible = Vec::new();
    let mut mark_visible = |pos: Pos| {
        let outside = (pos.1 as usize) >= tiles.len() || (pos.0 as usize) >= tiles[0].len();

        if !outside && !visible.contains(&pos) {
            visible.push(pos);
        }
    };

    compute_fov(origin, &mut is_blocking, &mut mark_visible);

    println!("{:?}", &visible);

    for y in 0..tiles.len() as isize {
        for x in 0..tiles[0].len() as isize {
            if visible.contains(&(x, y)) {
                print!("1");
            } else {
                print!("0");
            }
        }
        println!();
    }

    assert_eq!(tiles.len() * tiles[0].len(), visible.len());
}
