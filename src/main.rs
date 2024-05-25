use proconio::*;
use std::collections::{HashMap, HashSet};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[derive(Debug, Clone, PartialEq)]
struct ZobristHashSet<S: std::hash::Hash + Eq + Clone> {
    v: i64,
    x_to_hash: HashMap<S, i64>,
    rng: StdRng,
}

impl<S: std::hash::Hash + Eq + Clone> ZobristHashSet<S> {
    pub fn new() -> Self {
        let seed: [u8; 32] = rand::random();
        let rng = SeedableRng::from_seed(seed);
        ZobristHashSet {
            v: 0,
            x_to_hash: HashMap::new(),
            rng,
        }
    }

    pub fn flip(&mut self, x: S) {
        let hash_value = self.x_to_hash.entry(x.clone()).or_insert_with(|| {
            self.rng.gen_range(i64::MIN..=i64::MAX)
        });
        self.v ^= *hash_value;
    }

    pub fn init(&mut self) {
        self.v = 0;
    }

    pub fn get(&self) -> i64 {
        self.v
    }
}

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

enum Operation {
    Right,
    Down,
    Left,
    Up,
    Suspend,
    Lower,
    Stop,
    Explode,
}

const DIR_NUM: usize = 4;
const DX: [isize; DIR_NUM] = [0, 1, 0, -1];
const DY: [isize; DIR_NUM] = [1, 0, -1, 0];
const DIR: [char; DIR_NUM] = ['R', 'D', 'L', 'U'];

const OP_NUM: usize = 7;
const OP: [char; OP_NUM] = ['R', 'D', 'L', 'U', 'P', 'Q', '.'];

const MAX_WIDTH: usize = 10000;
const TURN: usize = 1000;
const USING_CRANE: usize = 2;

#[inline]
/* 反対の方向を返す関数 */
fn reverse_op(op: usize) -> Operation {
    match op {
        0 => Operation::Left,
        1 => Operation::Up,
        2 => Operation::Right,
        3 => Operation::Down,
        4 => Operation::Lower,
        5 => Operation::Suspend,
        6 => Operation::Stop,
        _ => panic!("invalid op"),
    }
}

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
    pre_op: usize,          // 1つ前の操作
    hash_set: HashSet<i64>, // 過去に存在した hash の集合
    zobrist: ZobristHashSet<(usize, usize, usize, i64, i64)>,
}
impl Crane {
    fn new(input: &Input, _idx: usize, _x: usize, _y: usize, _big: bool) -> Self {
        // Zobrist Hash で盤面重複を検出
        // ※ (x, y, suspended の bool, 真下の cont) の 4 つを key として扱い、重複検知
        let mut _zobrist = ZobristHashSet::new();
        let mut _hash_set: HashSet<i64> = HashSet::new();
        _zobrist.init();
        _zobrist.flip((_x, _y, 0, input.a[_x][0], Operation::Stop as i64));
        // (x, y, suspend の bool, 真下の cont, pre_op)
        _hash_set.insert(_zobrist.get());

        Self {
            h: input.n,
            w: input.n,
            x: _x,
            y: _y,
            idx: _idx,
            suspended: false,
            big: _big,
            exploded: false,
            pre_op: Operation::Stop as usize,
            hash_set: _hash_set,
            zobrist: _zobrist,
        }
    }

    #[inline]
    fn hash_flip(&mut self, grid_cont: &[Vec<Vec<i64>>]) {
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
    }

    fn shift(
        &mut self,
        dir: usize,
        grid_crane: &mut [Vec<isize>],
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
        self.hash_flip(grid_cont);
        let nx = (self.x as isize + DX[dir]) as usize;
        let ny = (self.y as isize + DY[dir]) as usize;
        grid_crane[nx][ny] = self.idx as isize;
        grid_crane[self.x][self.y] = -1;

        if self.suspended {
            grid_cont[nx][ny][self.big as usize] = grid_cont[self.x][self.y][self.big as usize];
            grid_cont[self.x][self.y][self.big as usize] = -1;
            cont_suspended[nx][ny][self.big as usize] = true;
            cont_suspended[self.x][self.y][self.big as usize] = false;
        }

        self.x = nx;
        self.y = ny;
        self.pre_op = dir;
        self.hash_flip(grid_cont);
        self.hash_set.insert(self.zobrist.get());
        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        DIR[dir]
    }

    fn suspend(
        &mut self,
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
        self.hash_flip(grid_cont);
        self.suspended = true;
        cont_suspended[self.x][self.y][self.big as usize] = true;
        if self.big {
            grid_cont[self.x][self.y].swap(0, 1);
        }
        self.pre_op = Operation::Suspend as usize;
        self.hash_flip(grid_cont);
        self.hash_set.insert(self.zobrist.get());

        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        'P'
    }

    fn lower(
        &mut self,
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
        self.hash_flip(grid_cont);
        self.suspended = false;
        cont_suspended[self.x][self.y][self.big as usize] = false;
        if self.big {
            grid_cont[self.x][self.y].swap(0, 1);
        }
        self.pre_op = Operation::Lower as usize;
        self.hash_flip(grid_cont);
        self.hash_set.insert(self.zobrist.get());
        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        'Q'
    }

    fn stop(&mut self, grid_cont: &mut [Vec<Vec<i64>>]) -> char {
        self.hash_flip(grid_cont);
        self.pre_op = Operation::Stop as usize;
        self.hash_flip(grid_cont);
        self.hash_set.insert(self.zobrist.get());
        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        '.'
    }

    fn explode(
        &mut self,
        grid_cont: &mut [Vec<Vec<i64>>],
        grid_crane: &mut [Vec<isize>],
    ) -> char {
        self.hash_flip(grid_cont);
        self.exploded = true;
        grid_crane[self.x][self.y] = -1;
        grid_crane[self.x][self.y] = -1;
        self.pre_op = Operation::Explode as usize;
        self.hash_flip(grid_cont);
        self.hash_set.insert(self.zobrist.get());
        'B'
    }

    fn action(
        &mut self,
        action: usize,
        grid_crane: &mut [Vec<isize>],
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
        match action {
            0..=3 => self.shift(action, grid_crane, grid_cont, cont_suspended),
            4 => self.suspend(grid_cont, cont_suspended),
            5 => self.lower(grid_cont, cont_suspended),
            6 => self.stop(grid_cont),
            7 => self.explode(grid_cont, grid_crane),
            _ => panic!("invalid action"),
        }
    }

    fn shift_ok(
        &mut self,
        dir: usize,
        grid_crane: &[Vec<isize>],
        grid_cont: &[Vec<Vec<i64>>],
        out_cont_idx: &[usize],
    ) -> bool {
        if self.exploded {
            // 既に爆破している場合は NG
            return false;
        }
        let nx = self.x as isize + DX[dir];
        let ny = self.y as isize + DY[dir];
        if out_field(nx, ny, self.h as isize, self.w as isize) {
            // フィールド外に出る場合は NG
            return false;
        }
        let nx = nx as usize;
        let ny = ny as usize;

        let pre_hash = self.zobrist.get();
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            nx,
            ny,
            self.suspended as usize,
            grid_cont[nx][ny][self.big as usize],
            dir as i64
        ));
        if self.hash_set.contains(&self.zobrist.get()) {
            // 既に同じ状態に遷移している場合は NG
            return false;
        }
        self.zobrist.flip((
            nx,
            ny,
            self.suspended as usize,
            grid_cont[nx][ny][self.big as usize],
            dir as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        assert!(pre_hash == self.zobrist.get());

        if grid_crane[nx][ny] != -1 {
            // 移動先にクレーンがいる場合は NG
            return false;
        }

        if !self.big && self.suspended && grid_cont[nx][ny][0] != -1 {
            // 小クレーンで吊り上げていて、移動先にコンテナがある場合は NG
            return false;
        }
        let mut flag: bool = false;
        for i in 0..self.h {
            flag |= out_cont_idx[i] as i64 == grid_cont[self.x][self.y][self.big as usize];
        }
        if self.suspended && ny == self.w - 1 && !flag {
            // 次に搬出すべきコンテナを持っていないのに、搬出口に移動しようとしている場合は NG
            return false;
        }

        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        true
    }

    fn suspend_ok(&mut self, grid_cont: &[Vec<Vec<i64>>]) -> bool {
        if self.exploded || self.suspended {
            // 既に爆破している場合や吊り上げている場合は NG
            return false;
        }

        let pre_hash = self.zobrist.get();
        self.zobrist.flip((
            self.x,
            self.y,
            false as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            true as usize,
            grid_cont[self.x][self.y][self.big as usize],
            Operation::Suspend as i64
        ));
        if self.hash_set.contains(&self.zobrist.get()) {
            // 既に同じ状態に遷移している場合は NG
            return false;
        }
        self.zobrist.flip((
            self.x,
            self.y,
            false as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            true as usize,
            grid_cont[self.x][self.y][self.big as usize],
            Operation::Suspend as i64
        ));
        assert!(pre_hash == self.zobrist.get());

        if grid_cont[self.x][self.y][0] == -1 {
            // 吊り上げるコンテナがない場合は NG
            return false;
        }

        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        true
    }

    fn lower_ok(
        &mut self,
        grid_cont: &[Vec<Vec<i64>>],
        out_cont_idx: &[usize]
    ) -> bool {
        if self.exploded || !self.suspended {
            // 既に爆破している場合や吊り上げていない場合は NG
            return false;
        }
        
        let pre_hash = self.zobrist.get();
        self.zobrist.flip((
            self.x,
            self.y,
            true as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            false as usize,
            grid_cont[self.x][self.y][self.big as usize],
            Operation::Suspend as i64
        ));
        if self.hash_set.contains(&self.zobrist.get()) {
            // 既に同じ状態に遷移している場合は NG
            return false;
        }
        self.zobrist.flip((
            self.x,
            self.y,
            true as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            false as usize,
            grid_cont[self.x][self.y][self.big as usize],
            Operation::Suspend as i64
        ));
        assert!(pre_hash == self.zobrist.get());

        if self.big && grid_cont[self.x][self.y][0] != -1 {
            // 大クレーンで吊り下げ中で、降ろす場所にコンテナがある場合は NG
            return false;
        }

        let cond_idx: usize = grid_cont[self.x][self.y][self.big as usize] as usize;
        let x = cond_idx / self.h;
        if !(self.y != self.w - 1 || self.x == x && out_cont_idx[x] == cond_idx) {
            // 降ろす場所が不適の場合は NG
            return false;
        }

        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        true
    }

    fn stop_ok(&mut self, grid_cont: &[Vec<Vec<i64>>]) -> bool {
        let pre_hash = self.zobrist.get();
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            Operation::Stop as i64
        ));
        if self.hash_set.contains(&self.zobrist.get()) {
            // 既に同じ状態に遷移している場合は NG
            return false;
        }
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            self.pre_op as i64
        ));
        self.zobrist.flip((
            self.x,
            self.y,
            self.suspended as usize,
            grid_cont[self.x][self.y][self.big as usize],
            Operation::Stop as i64
        ));
        assert!(pre_hash == self.zobrist.get());

        if self.suspended {
            assert!(grid_cont[self.x][self.y][0] != -1 || grid_cont[self.x][self.y][1] != -1);
        }
        true
    }

    fn explode_ok(&self) -> bool {
        // 既に爆破している場合は NG
        if self.exploded {
            return false;
        }
        true
    }

    fn action_ok(
        &mut self,
        action: usize,
        grid_crane: &[Vec<isize>],
        grid_cont: &[Vec<Vec<i64>>],
        out_cont_idx: &[usize],
    ) -> bool {
        match action {
            0..=3 => self.shift_ok(action, grid_crane, grid_cont, out_cont_idx),
            4 => self.suspend_ok(grid_cont),
            5 => self.lower_ok(grid_cont, out_cont_idx),
            6 => self.stop_ok(grid_cont),
            7 => self.explode_ok(),
            _ => panic!("invalid action"),
        }
    }
}

#[derive(Clone,PartialEq)]
struct Terminal {
    h: usize,
    w: usize,
    score: i64,
    turn: usize,
    hash: u64,                           // Zobrist Hash
    conts: Vec<Vec<i64>>,                // 行 i から j 番目に来るコンテナの index
    out_cont_idx: Vec<usize>,            // 各搬出口から今搬出すべきコンテナの index
    incoming_cont_idx: Vec<usize>,       // 各搬入口から今搬入すべきコンテナの index
    grid_cont: Vec<Vec<Vec<i64>>>,       // (i, j) で上空(1) or 接地(0) が k のコンテナの index
    grid_crane: Vec<Vec<isize>>,         // (i, j) にいるクレーンの index
    cranes: Vec<Crane>,                  // 各クレーンの情報
    cont_suspended: Vec<Vec<Vec<bool>>>, // (i, j) にあるコンテナが吊り上げられているか

    out_cont_turn: Vec<Vec<(usize, i64)>>, // 各搬出口から i ターン目に搬出したコンテナの (index, x)
    incoming_cont_turn: Vec<Vec<usize>>,     // 各搬入口から i ターン目に搬入したコンテナの (index, x)
}
impl Terminal {
    fn new(input: &Input) -> Self {
        // 搬出するコンテナの index 初期化
        let mut _out_cont_idx: Vec<usize> = vec![0; input.n];
        for (i, cont_idx) in _out_cont_idx.iter_mut().enumerate() {
            *cont_idx = i * input.n;
        }

        // クレーンをターミナル上で初期化
        let mut _cranes: Vec<Crane> = vec![];
        let mut _grid_crane: Vec<Vec<isize>> = vec![vec![-1; input.n]; input.n];
        for (i, crane) in _grid_crane.iter_mut().enumerate().take(input.n) {
            let big = i == 0;
            _cranes.push(Crane::new(input, i, i, 0, big));
            crane[0] = i as isize;
        }

        Self {
            h: input.n,
            w: input.n,
            score: 0,
            turn: 0,
            hash: 0,
            conts: input.a.to_vec(),
            out_cont_idx: _out_cont_idx,
            incoming_cont_idx: vec![0; input.n],
            grid_cont: vec![vec![vec![-1; 2]; input.n]; input.n],
            grid_crane: _grid_crane,
            cranes: _cranes,
            cont_suspended: vec![vec![vec![false; 2]; input.n]; input.n],
            out_cont_turn: vec![vec![]; TURN+1],
            incoming_cont_turn: vec![vec![]; TURN+1],
        }
    }

    /* 搬入口が空いてる時にコンテナを搬入する関数 */
    fn prepare_cont(&mut self) {
        for i in 0..self.h {
            // 搬入口が空いている場合
            if self.grid_cont[i][0][0] == -1 && self.grid_cont[i][0][1] == -1 && self.incoming_cont_idx[i] < self.w {
                // コンテナを搬入
                self.grid_cont[i][0][0] = self.conts[i][self.incoming_cont_idx[i]];
                // そのターンに何を搬入したかを履歴として持つ
                self.incoming_cont_turn[self.turn].push(i);
                // 次に搬入すべきコンテナに更新
                self.incoming_cont_idx[i] += 1;
            }
        }
    }

    /* 今のターンで搬入口から搬入されたコンテナを元に戻す関数 */
    fn prepare_cont_revert(&mut self) {
        for i in self.incoming_cont_turn[self.turn].clone() {
            // 搬入したコンテナをクリア
            self.grid_cont[i][0][0] = -1;
            // 次に搬入すべきコンテナを戻す
            self.incoming_cont_idx[i] -= 1;
        }
        self.incoming_cont_turn[self.turn].clear();
    }

    /* 搬出口にあるコンテナを搬出する関数 */
    fn carry_out_cont(&mut self) {
        for i in 0..self.h {
            // コンテナ搬出
            if self.grid_cont[i][self.w-1][0] != -1 && !self.cont_suspended[i][self.w-1][0] {
                // そのターンに何を搬出したかを履歴として持つ
                self.out_cont_turn[self.turn].push((i, self.grid_cont[i][self.w-1][0]));
                // 次に搬出すべきコンテナに更新
                self.out_cont_idx[i] += 1;
                // 搬出済みなのでコンテナ情報をクリア
                self.grid_cont[i][self.w-1][0] = -1;
            }
        }
    }

    /* 今のターンで搬出口から搬出されたコンテナを元に戻す関数 */
    fn carry_out_cont_revert(&mut self) {
        // for i in self.out_cont_turn[self.turn].clone() {
        for (i, cont_id) in self.out_cont_turn[self.turn].clone() {
            // 搬出したコンテナをクリア
            self.grid_cont[i][self.w-1][0] = cont_id;
            // 次に搬出すべきコンテナを戻す
            self.out_cont_idx[i] -= 1;
        }
        self.out_cont_turn[self.turn].clear();
    }

    /* 次のノードに遷移する関数 */
    fn apply(&mut self, node: &Node) {
        let action = node.op;

        self.cranes[self.turn % USING_CRANE].action(
            action,
            &mut self.grid_crane,
            &mut self.grid_cont,
            &mut self.cont_suspended,
        );
        self.carry_out_cont();
        self.prepare_cont();
        self.turn += 1;
        
        self.score = self.evaluate();

        // eprintln!("===== apply =====");
        // eprintln!("turn: {}, score: {}, op: {}", self.turn, self.score, OP[action]);
        // for i in 0..self.h {
        //     eprint!("{} ", self.incoming_cont_idx[i]);
        // }
        // eprintln!();
        // for i in 0..self.h {
        //     for j in 0..self.w {
        //         eprint!("{} ", self.grid_cont[i][j][0]);
        //     }
        //     eprintln!();
        // }
        // eprintln!();
        // for i in 0..self.h {
        //     for j in 0..self.w {
        //         eprint!("{} ", self.grid_crane[i][j]);
        //     }
        //     eprintln!();
        // }
        // eprintln!("=================")
    }

    /* 前のノードに遷移する関数 */
    fn revert(&mut self, node: &Node) {
        let action = node.op;

        self.turn -= 1;
        self.prepare_cont_revert();
        self.carry_out_cont_revert();
        self.cranes[self.turn % USING_CRANE].action(
            reverse_op(action) as usize,
            &mut self.grid_crane,
            &mut self.grid_cont,
            &mut self.cont_suspended,
        );

        self.score = self.evaluate();

        // eprintln!("===== revert =====");
        // eprintln!("turn: {}, score: {}, op: {}", self.turn, self.score, OP[action]);
        // for i in 0..self.h {
        //     eprint!("{} ", self.incoming_cont_idx[i]);
        // }
        // eprintln!();
        // for i in 0..self.h {
        //     for j in 0..self.w {
        //         eprint!("{} ", self.grid_cont[i][j][0]);
        //     }
        //     eprintln!();
        // }
        // eprintln!();
        // for i in 0..self.h {
        //     for j in 0..self.w {
        //         eprint!("{} ", self.grid_crane[i][j]);
        //     }
        //     eprintln!();
        // }
        // eprintln!("=================")
    }

    fn evaluate(&self) -> i64 {
        /*
        ========== 評価関数 ==========
        目的地点とコンテナのマンハッタン距離を d として、∑_{i,j} d(i,j) * 10^{何番目に搬出すべきか} の最小化を目指す
        */
        // 盤面に存在するコンテナの距離を考慮
        let mut score: i64 = 0;
        for i in 0..self.h {
            for j in 0..self.w {
                for k in 0..2 {
                    let gx = self.grid_cont[i][j][k] / self.h as i64;
                    let perm = self.grid_cont[i][j][k] % self.h as i64;
                    if self.grid_cont[i][j][k] != -1 {
                        let mut add = 0;
                        // x 方向の寄与
                        add += (i as i64 - gx).abs();
                        // y 方向の寄与
                        add += (j as i64 - (self.w - 1) as i64).abs();
                        // 倍率
                        add *= 10_i64.pow(2 * (self.w as i64 - perm - 1) as u32 + 1);
                        score += add;
                    }
                }
            }
        }
        // 盤面に存在しないコンテナの距離を考慮
        for i in 0..self.h {
            for j in self.incoming_cont_idx[i]..self.w {
                let gx = self.conts[i][j] / self.h as i64;
                let perm = self.conts[i][j] % self.h as i64;

                let mut add = 0;
                // x 方向の寄与
                add += (i as i64 - gx).abs();
                // y 方向の寄与
                add += ( -(j as i64 - self.incoming_cont_idx[i] as i64 + 1) - (self.w - 1) as i64).abs();
                // 倍率
                add *= 10_i64.pow(2 * (self.w as i64 - perm - 1) as u32);
                score += add;
            }
        }
        score
    }
}

#[derive(Clone, Debug)]
struct Cand {
    op: usize,
    parent: usize,
    eval_score: i64,
    hash: u64,
}
impl Cand {
    fn raw_score(&self) -> i64 {
        todo!();
    }
    
    fn to_node(&self) -> Node {
        Node{
            child: !0,
            prev: !0,
            next: !0,
            op: self.op,
            parent: self.parent,
        }
    }
}


#[derive(Clone,Default)]
struct Node {
    op: usize,
    parent: usize, // 親Node
    child: usize, // 代表の子Node
    prev: usize, // 前の兄弟Node
    next: usize, // 次の兄弟Node
}

struct BeamSearch {
    state: Terminal,
    leaf: Vec<usize>, // 子が存在しないNodeのindex
    next_leaf: Vec<usize>,
    nodes: Vec<Node>,
    cur_node: usize,
    free: Vec<usize>, // nodesのうち使われていないindex
}
impl BeamSearch {
    fn new(state: Terminal, node: Node) -> BeamSearch {
        const MAX_NODES:usize = MAX_WIDTH * TURN;
        let mut nodes = vec![Node::default();MAX_NODES];
        nodes[0] = node;
        let free=(1..MAX_NODES).rev().collect();
        
        BeamSearch {
            state,nodes,free,
            leaf:vec![0],
            next_leaf:vec![],
            cur_node:0,
        }
    }
    
    // 頂点を新たに追加する
    // 代表の子Nodeの前に挿入する形で実装
    fn add_node(&mut self, cand: Cand) {
        let next=self.nodes[cand.parent].child;
        let new=self.free.pop().expect("MAX_NODEが足りないよ");
        if next != !0 {
            self.nodes[next].prev=new;
        }
        self.nodes[cand.parent].child=new;
        
        self.next_leaf.push(new);
        self.nodes[new] = Node {next,..cand.to_node()};
    }

    // 既に探索済みのノードで葉のノードを再帰的に消していく
    fn del_node(&mut self, mut idx: usize) {
        loop {
            self.free.push(idx);
            let Node {prev,next,parent,..} = self.nodes[idx];
            assert_ne!(parent,!0,"全てのノードを消そうとしています");
            // 兄弟がいないなら親を消しに行く
            if prev&next== !0 {
                idx=parent;
                continue;
            }

            if prev != !0 {
                self.nodes[prev].next=next;
            }
            else {
                self.nodes[parent].child=next;
            }
            if next != !0 {
                self.nodes[next].prev=prev;
            }
            
            break;
        }
    }

        // dfsで木を走査
    // 一本道の場合戻る必要はないのでそれをsingleで管理
    fn dfs(&mut self, cands: &mut Vec<Cand>, single: bool){
        if self.nodes[self.cur_node].child==!0{
            self.append_cands(self.cur_node,cands);
            return;
        }

        let node=self.cur_node;
        let mut child=self.nodes[node].child;
        let next_single=single&(self.nodes[child].next==!0);

        // let prev_state=self.state.clone();
        loop{
            self.cur_node=child;
            self.state.apply(&self.nodes[child]);
            self.dfs(cands,next_single);

            if !next_single{
                self.state.revert(&self.nodes[child]);
                // assert!(prev_state==self.state);
            }
            child=self.nodes[child].next;
            if child==!0{
                break;
            }
        }
        
        if !next_single{
            self.cur_node=node;
        }
    }

    // 走査の非再帰実装
    fn no_dfs(&mut self, cands: &mut Vec<Cand>) {
        // 1本道でなくなるまで潜る
        loop {
            let Node {next, child,..} = self.nodes[self.cur_node];
            if next == !0 || child == !0 {
                break;
            }
            self.cur_node = child;
            self.state.apply(&self.nodes[self.cur_node]);
        }

        let root=self.cur_node;
        loop {
            let child = self.nodes[self.cur_node].child;
            if child == !0 {
                self.append_cands(self.cur_node, cands);
                loop {
                    if self.cur_node == root {
                        return;
                    }
                    let node = &self.nodes[self.cur_node];
                    self.state.revert(node);
                    if node.next != !0 {
                        self.cur_node = node.next;
                        self.state.apply(&self.nodes[self.cur_node]);
                        break;
                    }
                    self.cur_node = node.parent;
                }
            }
            else {
                self.cur_node = child;
                self.state.apply(&self.nodes[self.cur_node]);
            }
        }
    }

    fn enum_cands(&mut self, cands: &mut Vec<Cand>) {
        // self.dfs(cands, true);
        self.no_dfs(cands);
    }

    fn update<I:Iterator<Item=Cand>>(&mut self, cands:I) {
        self.next_leaf.clear();
        for cand in cands {
            self.add_node(cand);
        }

        for i in 0..self.leaf.len() {
            let n = self.leaf[i];
            // 子が存在しないノードは無駄なので消す
            if self.nodes[n].child == !0 {
                self.del_node(n);
            }
        }

        std::mem::swap(&mut self.leaf,&mut self.next_leaf);
    }

    fn restore(&self, mut idx: usize) -> Vec<usize> {
        let mut ret=vec![];
        loop {
            let Node {op,parent,..} = self.nodes[idx];
            if op == !0 {
                break;
            }
            ret.push(op);
            idx = parent;
        }
        
        ret.reverse();
        ret
    }

    // self.stateがself.nodes[idx]のノードが表す状態になっている
    // self.nodes[idx]からのCandをcandsに積む
    fn append_cands(&mut self, idx: usize, cands: &mut Vec<Cand>) {
        let node = &self.nodes[idx];
        assert_eq!(node.child, !0);

        for _op in 0..OP_NUM {
            // 前回の逆操作は無視
            if reverse_op(_op) as usize == self.state.cranes[self.state.turn % USING_CRANE].pre_op {
                continue;
            }

            // 行動可能かを check
            if !self.state.cranes[self.state.turn % USING_CRANE].action_ok(
                _op,
                &self.state.grid_crane,
                &self.state.grid_cont,
                &self.state.out_cont_idx,
            ) {
                continue;
            }

            // 盤面評価値を計算（差分計算で求める）
            let mut _eval_score = self.state.score;
            if _op < 4 && self.state.cranes[self.state.turn % USING_CRANE].suspended {
                let crane = &self.state.cranes[self.state.turn % USING_CRANE];

                let px = crane.x as i64;
                let py = crane.y as i64;
                let nx = crane.x as i64 + DX[_op] as i64;
                let ny = crane.y as i64 + DY[_op] as i64;

                let cont = self.state.grid_cont[px as usize][py as usize][crane.big as usize];
                // eprintln!("px: {}, py: {}, nx: {}, ny: {}, cont: {}, op: {}, crane_id: {}", px, py, nx, ny, cont, _op, self.state.turn % USING_CRANE);
                // eprintln!("cont_under: {}, cont_upper: {}", self.state.grid_cont[nx as usize][ny as usize][0], self.state.grid_cont[nx as usize][ny as usize][1]);
                assert!(cont != -1);
                let gx = cont / self.state.h as i64;
                let perm = cont % self.state.h as i64;

                _eval_score -= ((px - gx).abs() + (py - (self.state.w - 1) as i64).abs()) * 10_i64.pow(2 * (self.state.w as i64 - perm - 1) as u32);
                _eval_score += ((nx - gx).abs() + (ny - (self.state.w - 1) as i64).abs()) * 10_i64.pow(2 * (self.state.w as i64 - perm - 1) as u32);
            }

            cands.push(
                Cand {
                    op: _op,
                    parent: idx,
                    eval_score: _eval_score,
                    hash: 0,
                }
            );
        }
    }
}

fn write_output(actions: String) {
    // ÷5 がターン数、mod 5 がクレーン i の出力
    let mut ans: Vec<String> = vec!["".to_string(); 5];
    for (i, action) in actions.chars().enumerate() {
        ans[i % USING_CRANE].push(action);
    }
    for a in ans {
        println!("{}", a);
    }
}

fn main() {
    let input = Input::read_input();

    /*  ========== ビームサーチ解法 ========== */
    // BeamSearchの初期化
    let mut initial_terminal = Terminal::new(&input);
    initial_terminal.prepare_cont();
    initial_terminal.incoming_cont_turn[0].clear();
    initial_terminal.score = initial_terminal.evaluate();
    let initial_node: Node = {
        Node {
            op: !0,
            parent: !0,
            child: !0,
            prev: !0,
            next: !0,
        }
    };
    // USING_CRANE 以降は初手爆破
    for i in USING_CRANE..input.n {
        initial_terminal.cranes[i].explode(&mut initial_terminal.grid_cont, &mut initial_terminal.grid_crane);
    }
    let mut solver = BeamSearch::new(
        initial_terminal,
        initial_node,
    );

    for turn in 0..TURN/5 {
        eprintln!("turn: {}", turn);

        // 候補リストを生成
        let mut cands = Vec::new();
        solver.enum_cands(&mut cands);

        // スコアが低い順にソート (reverse)
        cands.sort_by(|a, b| a.eval_score.cmp(&b.eval_score));
        let top_cands = cands.into_iter().take(MAX_WIDTH);

        // top_cands の top 3 の候補を表示
        let _top_cands: Vec<Cand> = top_cands.clone().collect();
        for cand in _top_cands.iter().take(3) {
            eprintln!("Score: {}, op: {}", cand.eval_score, OP[cand.op]);
        }
        eprintln!("candidates: {}\n", solver.leaf.len());

        // 最も良いスコアが 0 になった場合に終了
        if _top_cands[0].eval_score == 0 {
            solver.cur_node = _top_cands[0].parent;
            break;
        }
        
        // 候補を基に次の状態を更新
        solver.update(top_cands);

        // if turn == 18 {
        //     let path = solver.restore(solver.leaf[0]);
        //     let mut actions: String = "".to_string();
        //     eprintln!("path: {:?}", path);
        //     for op in path {
        //         actions.push(OP[op]);
        //     }
        //     write_output(actions);
        // }
    }

    // 最終状態の復元
    solver.cur_node = solver.leaf[0];
    let final_path = solver.restore(solver.cur_node);
    let mut actions: String = "".to_string();
    eprintln!("path: {:?}", final_path);
    for op in final_path {
        actions.push(OP[op]);
    }

    write_output(actions)
}