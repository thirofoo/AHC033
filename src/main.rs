use proconio::*;

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

#[derive(Clone,PartialEq)]
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
    fn new(input: &Input, _idx: usize, _x: usize, _y: usize, _big: bool) -> Self {
        Self {
            h: input.n,
            w: input.n,
            x: _x,
            y: _y,
            idx: _idx,
            suspended: false,
            big: _big,
            exploded: false,
        }
    }

    fn shift(
        &mut self,
        dir: usize,
        grid_crane: &mut [Vec<isize>],
        grid_cont: &mut [Vec<Vec<i64>>],
    ) -> char {
        if self.exploded {
            // クレーンが爆発している場合は NG
            return 'X'; // 行動失敗時は 'X' を返す
        }
        let nx = self.x as isize + DX[dir];
        let ny = self.y as isize + DY[dir];

        if out_field(nx, ny, self.h as isize, self.w as isize) {
            // 場外に出ている場合は NG
            return 'X';
        }
        let nx = nx as usize;
        let ny = ny as usize;

        if grid_crane[nx][ny] != -1 {
            // 隣にクレーンがある場合は NG
            return 'X';
        }

        if self.suspended && grid_cont[nx][ny][self.big as usize] != -1 {
            // 吊り下げていて移動先にコンテナがある場合は NG
            return 'X';
        }

        grid_crane[nx][ny] = self.idx as isize;
        grid_crane[self.x][self.y] = -1;

        if self.suspended {
            let next_cont = grid_cont[self.x][self.y][self.big as usize];
            grid_cont[nx][ny][self.big as usize] = next_cont;
            grid_cont[self.x][self.y][self.big as usize] = -1;
        }

        self.x = nx;
        self.y = ny;
        DIR[dir]
    }

    fn suspend(&mut self, grid_cont: &mut [Vec<Vec<i64>>]) -> char {
        if self.exploded || self.suspended {
            // クレーンが爆発していたり、すでに吊り下げている場合は NG
            return 'X';
        }
        self.suspended = true;
        if self.big {
            let next_cont = grid_cont[self.x][self.y][0];
            grid_cont[self.x][self.y][1] = next_cont;
            grid_cont[self.x][self.y][0] = -1;
        }
        'P'
    }

    fn lower(&mut self, grid_cont: &mut [Vec<Vec<i64>>]) -> char {
        if self.exploded || !self.suspended {
            // クレーンが爆発していたり、吊り下げていない場合は NG
            return 'X';
        }
        self.suspended = false;
        if self.big {
            let next_cont = grid_cont[self.x][self.y][1];
            grid_cont[self.x][self.y][0] = next_cont;
            grid_cont[self.x][self.y][1] = -1;
        }
        'Q'
    }

    fn stop(&self) -> char {
        '.'
    }

    fn explode(&mut self, grid_crane: &mut [Vec<isize>]) -> char {
        if self.exploded {
            // クレーンが爆発している場合は NG
            return 'X';
        }
        self.exploded = true;
        grid_crane[self.x][self.y] = -1;
        grid_crane[self.x][self.y] = -1;
        'B'
    }
}

#[derive(Clone,PartialEq)]
struct Terminal {
    h: usize,
    w: usize,
    conts: Vec<Vec<i64>>,          // 行 i から j 番目に来るコンテナの index
    out_cont_idx: Vec<usize>,      // 各搬出口から今搬出すべきコンテナの index
    incoming_cont_idx: Vec<usize>, // 各搬入口から次に場に出るべきコンテナの index
    grid_cont: Vec<Vec<Vec<i64>>>, // (i, j) で上空(1) or 接地(0) が k のコンテナの index
    grid_crane: Vec<Vec<isize>>,   // (i, j) にいるクレーンの index
    cranes: Vec<Crane>,            // 各クレーンの情報
}

impl Terminal {
    fn new(input: &Input) -> Self {
        let mut _out_cont_idx: Vec<usize> = vec![0; input.n];
        // 搬出するコンテナの index 初期化
        for (i, cont_idx) in _out_cont_idx.iter_mut().enumerate() {
            *cont_idx = i * input.n;
        }
        // クレーンをターミナル上で初期化
        let mut _grid_crane: Vec<Vec<isize>> = vec![vec![-1; input.n]; input.n];
        let mut _cranes: Vec<Crane> = vec![];
        for i in 0..input.n {
            let big = i == 0;
            _cranes.push(Crane::new(input, i, i, 0, big));
        }
        for (i, crane) in _cranes.iter().enumerate() {
            _grid_crane[crane.x][crane.y] = i as isize;
        }
        Self {
            h: input.n,
            w: input.n,
            conts: input.a.to_vec(),
            out_cont_idx: _out_cont_idx,
            incoming_cont_idx: vec![0; input.n],
            grid_cont: vec![vec![vec![-1; 2]; input.n]; input.n],
            grid_crane: _grid_crane,
            cranes: _cranes,
        }
    }

    /* 搬入口が空いてる時にコンテナを展開する関数 */
    fn prepare_cont(&mut self) {
        for i in 0..self.h {
            // コンテナ展開
            if self.grid_cont[i][0][0] == -1 && self.incoming_cont_idx[i] < self.conts.len() {
                self.grid_cont[i][0][0] = self.conts[i][self.incoming_cont_idx[i]];
                self.incoming_cont_idx[i] += 1;
            }
        }
    }

    /* 搬出口にあるコンテナを搬出する関数 */
    fn carry_out_cont(&mut self) {
        for i in 0..self.h {
            // コンテナ搬出
            if self.grid_cont[i][self.w-1][0] != -1 {
                let incoming_cont_idx = self.grid_cont[i][self.w-1][0];
                self.grid_cont[i][self.w-1][0] = -1;
                self.out_cont_idx[(incoming_cont_idx / self.h as i64) as usize] += 1;
            }
        }
    }

    /* 差分更新で O(1) で次のノードに遷移する関数 */
    fn apply(&mut self, node: &Node){
        todo!();
    }

    /* 差分更新で O(1) で前のノードに遷移する関数 */
    fn revert(&mut self,node: &Node){
        todo!();
    }
}

struct Node {
    action: char,
    crane_idx: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

fn write_output(actions: String) {
    // ÷5 がターン数、mod 5 がクレーン i の出力
    let mut ans: Vec<String> = vec!["".to_string(); 5];
    for (i, action) in actions.chars().enumerate() {
        ans[i % 5].push(action);
    }
    for a in ans {
        println!("{}", a);
    }
}

fn main() {
    // 答えとなる行動の文字列を 1 次元化したもの
    let mut actions: String = "".to_string();

    let input = Input::read_input();
    let mut terminal = Terminal::new(&input);
    terminal.prepare_cont(); // 初期コンテナの展開

    /* 1. 20 マス分コンテナを展開する */
    for y in (1..input.n-1).rev() {
        // コンテナ吊り上げ
        for crane in terminal.cranes.iter_mut() {
            let action = crane.suspend(&mut terminal.grid_cont);
            actions.push_str(&action.to_string());
        }
        // 右移動
        for _ in 0..y {
            for crane in terminal.cranes.iter_mut() {
                let action = crane.shift(
                    Direction::Right as usize,
                    &mut terminal.grid_crane,
                    &mut terminal.grid_cont
                );
                actions.push_str(&action.to_string());
            }
        }
        // コンテナ下ろし
        for crane in terminal.cranes.iter_mut() {
            let action = crane.lower(&mut terminal.grid_cont);
            actions.push_str(&action.to_string());
        }
        // 左移動
        for _ in 0..y {
            for crane in terminal.cranes.iter_mut() {
                let action = crane.shift(
                    Direction::Left as usize,
                    &mut terminal.grid_crane,
                    &mut terminal.grid_cont
                );
                actions.push_str(&action.to_string());
            }
        }
    }

    /* 2. 小クレーンを右端に移動 */
    for _ in 0..input.n-1 {
        for crane in terminal.cranes.iter_mut() {
            let action = crane.shift(
                Direction::Right as usize,
                &mut terminal.grid_crane,
                &mut terminal.grid_cont
            );
            actions.push_str(&action.to_string());
        }
    }
    write_output(actions)
}