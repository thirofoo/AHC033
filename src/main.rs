use proconio::*;
use rand::prelude::*;
use itertools::Itertools;
use std::{cmp::Reverse, collections::BinaryHeap, process::exit};

pub fn get_time() -> f64 {
    static mut STIME: f64 = -1.0;
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let ms = t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9;
    unsafe {
        if STIME < 0.0 {
            STIME = ms;
        }
        /* ローカル環境とジャッジ環境の実行速度差はget_timeで吸収しておくと便利 */
        #[cfg(feature = "local")]
        {
            (ms - STIME) * 0.8
        }
        #[cfg(not(feature = "local"))]
        {
            ms - STIME
        }
    }
}
/* ⇓ ========== ここから本実装 ========== ⇓ */

/*  right | down | left | up */
enum Direction {
    Right,
    Down,
    Left,
    Up,
}
const DIR_NUM: usize = 4;
const DX: [isize; DIR_NUM] = [0, 1, 0, -1];
const DY: [isize; DIR_NUM] = [1, 0, -1, 0];
const DIR: [char; DIR_NUM] = ['R', 'D', 'L', 'U'];

#[inline]
fn out_field(x: isize, y: isize, h: isize, w: isize) -> bool {
    !(0 <= x && x < h && 0 <= y && y < w)
}

struct Input {
    n: usize,
    a: Vec<Vec<i64>>,
}

impl Input {
    fn read_input() -> Self {
        input! {
            n: usize,
            a: [[i64; n]; n],
        }
        Self { n, a }
    }
}

struct Terminal {
    h: usize,
    w: usize,
    grid: Vec<Vec<Vec<i64>>>,
    containers: Vec<Vec<i64>>,
    container_idx: Vec<usize>,
    cranes_place: Vec<Vec<isize>>,
}

impl Terminal {
    fn new(h: usize, w: usize, containers: Vec<Vec<i64>>) -> Self {
        let grid: Vec<Vec<Vec<i64>>> = vec![vec![vec![-1; 2]; w]; h];
        let container_idx: Vec<usize> = vec![0; h];
        let mut cranes_place: Vec<Vec<isize>> = vec![vec![-1; w]; h];
        Self { h, w, grid, containers, container_idx, cranes_place }
    }

    fn expand_container(&mut self) {
        /* 盤面の左端が空きの場合にコンテナを展開する関数 */
        for i in 0..self.h {
            if self.grid[i][0][0] == -1 && self.container_idx[i] < self.containers.len() {
                self.grid[i][0][0] = self.containers[i][self.container_idx[i]];
                self.container_idx[i] += 1;
            }
        }
    }
}

struct Crane {
    h: usize,
    w: usize,
    x: usize,
    y: usize,
    idx: usize,
    suspended: bool,
    big: bool,
    exploded: bool,
}

impl Crane {
    fn new(h: usize, w: usize, x: usize, y: usize, idx: usize, big: bool, terminal: &mut Terminal) -> Self {
        assert!(terminal.cranes_place[x][y] == -1);
        terminal.cranes_place[x][y] = idx as isize;
        Self { h, w, x, y, idx, suspended: false, big, exploded: false }
    }

    fn shift(&mut self, dir: usize, terminal: &mut Terminal) -> char {
        assert!(!self.exploded);
        let nx = self.x as isize + DX[dir];
        let ny = self.y as isize + DY[dir];
        eprintln!("nx: {}, ny: {}", nx, ny);

        /* 場外に出ていないかを確認 */
        assert!(!out_field(nx, ny, self.h as isize, self.w as isize));
        let nx = nx as usize;
        let ny = ny as usize;

        /* 隣にクレーンが無いことを確認 */
        assert!(terminal.cranes_place[nx][ny] == -1);

        /* suspend している場合に移動先にコンテナが無いかを確認 */
        if self.suspended {
            assert!(terminal.grid[nx][ny][self.big as usize] == -1);
        }

        terminal.cranes_place[nx][ny] = terminal.cranes_place[self.x][self.y];
        terminal.cranes_place[self.x][self.y] = -1;
        if self.suspended {
            terminal.grid[nx][ny][self.big as usize] = terminal.grid[self.x][self.y][self.big as usize];
            terminal.grid[self.x][self.y][self.big as usize] = -1;
        }

        self.x = nx;
        self.y = ny;
        DIR[dir]
    }

    fn suspend(&mut self, terminal: &mut Terminal) -> char {
        assert!(!self.exploded);
        assert!(!self.suspended);
        self.suspended = true;
        if self.big {
            terminal.grid[self.x][self.y][1] = terminal.grid[self.x][self.y][0];
            terminal.grid[self.x][self.y][0] = -1;
        }
        'P'
    }

    fn lower(&mut self, terminal: &mut Terminal) -> char {
        assert!(!self.exploded);
        assert!(self.suspended);
        self.suspended = false;
        if self.big {
            terminal.grid[self.x][self.y][0] = terminal.grid[self.x][self.y][1];
            terminal.grid[self.x][self.y][1] = -1;
        }
        'Q'
    }

    fn stop(&self) -> char {
        '.'
    }

    fn explode(&mut self, terminal: &mut Terminal) -> char {
        assert!(!self.exploded);
        self.exploded = true;
        terminal.grid[self.x][self.y][0] = -1;
        terminal.grid[self.x][self.y][1] = -1;
        terminal.cranes_place[self.x][self.y] = -1;
        terminal.cranes_place[self.x][self.y] = -1;
        'X'
    }
}

fn write_output(out: Action) {
    for log in out.log {
        println!("{}", log);
    }
}

struct Action {
    log: Vec<String>
}

impl Action {
    fn new(size: usize) -> Self {
        Self {
            log: vec!["".to_string(); size]
        }
    }

    fn push(&mut self, idx: usize, step: char) {
        self.log[idx].push(step);
    }
}

fn main() {
    /*
    ========== 貪欲解法 ==========
    1. 20 マス分コンテナを展開する (残り 5 個が 0, 5, 10, 15, 20 のケースは一旦考えない)
    2. 小クレーンを右端に移動
    3. 大クレーンで今現在正しく運び出せるコンテナを右端に移動
      - この時、小クレーンが邪魔になる場合は、右端の上下で上手く移動させる
    4. 4, 9, 14, 19 のコンテナが場にある場合は、空きマスと大クレーンを用いて小クレーンの前に移動させる
    5. 小クレーンの前に 4, 9, 14, 19 のコンテナがある場合は、小クレーンに吊り下げる
    6. 大クレーンで右端に詰める
    7. 4, 9, 14, 19 以外のコンテナが積み終わるまで 3 ~ 6 を繰り返す
    */

    let input = Input::read_input();
    let mut terminal = Terminal::new(input.n, input.n, input.a);
    terminal.expand_container();
    let mut cranes: Vec<Crane> = vec![];
    for i in 0..input.n {
        let big = i == 0;
        cranes.push(Crane::new(input.n, input.n, i, 0, i, big, &mut terminal));
    }
    let mut actions = Action::new(input.n);

    eprintln!("terminal.container_idx: {:?}", terminal.container_idx);
    eprintln!("terminal.cranes_place: {:?}", terminal.cranes_place);

    /* 1. 20 マス分コンテナを展開する */
    for y in (1..input.n-1).rev() {
        for (i, crane) in cranes.iter_mut().enumerate() {
            actions.push(i, crane.suspend(&mut terminal));
            for _ in 0..y {
                actions.push(i, crane.shift(Direction::Right as usize, &mut terminal));
            }
            actions.push(i, crane.lower(&mut terminal));

            if y == 1 {
                /* 最後は左端に戻らなくてよい */
                break;
            }
            for _ in 0..y {
                actions.push(i, crane.shift(Direction::Left as usize, &mut terminal));
            }
        }
    }
    
    write_output(actions);
}