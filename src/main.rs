use proconio::*;
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

enum Operation {
    Right,
    Down,
    Left,
    Up,
    Suspend,
    Lower,
    Stop,
}

const DIR_NUM: usize = 4;
const DX: [isize; DIR_NUM] = [0, 1, 0, -1];
const DY: [isize; DIR_NUM] = [1, 0, -1, 0];
const DIR: [char; DIR_NUM] = ['R', 'D', 'L', 'U'];

const OP_NUM: usize = 7;
const OP: [char; OP_NUM] = ['R', 'D', 'L', 'U', 'P', 'Q', '.'];

const MAX_WIDTH: usize = 10000;
const TURN: usize = 1000;
const USING_CRANE: usize = 5;
const CRANE_PERM: [usize; USING_CRANE] = [0, 1, 2, 3, 4];

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
    pre_op: usize,          // 1つ前の操作
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
            pre_op: !0,
        }
    }

    fn shift(
        &mut self,
        dir: usize,
        grid_crane: &mut [Vec<isize>],
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
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
        DIR[dir]
    }

    fn suspend(
        &mut self,
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
        self.suspended = true;
        cont_suspended[self.x][self.y][self.big as usize] = true;
        if self.big {
            assert!(grid_cont[self.x][self.y][0] != -1);
            grid_cont[self.x][self.y].swap(0, 1);
        }
        self.pre_op = Operation::Suspend as usize;
        'P'
    }

    fn lower(
        &mut self,
        grid_cont: &mut [Vec<Vec<i64>>],
        cont_suspended: &mut [Vec<Vec<bool>>],
    ) -> char {
        self.suspended = false;
        cont_suspended[self.x][self.y][self.big as usize] = false;
        if self.big {
            assert!(grid_cont[self.x][self.y][1] != -1);
            grid_cont[self.x][self.y].swap(0, 1);
        }
        self.pre_op = Operation::Lower as usize;
        'Q'
    }

    fn stop(&mut self, grid_cont: &mut [Vec<Vec<i64>>]) -> char {
        self.pre_op = Operation::Stop as usize;
        '.'
    }

    fn explode(
        &mut self,
        grid_cont: &mut [Vec<Vec<i64>>],
        grid_crane: &mut [Vec<isize>],
    ) -> char {
        grid_crane[self.x][self.y] = -1;
        grid_crane[self.x][self.y] = -1;
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
        let nx = self.x as isize + DX[dir];
        let ny = self.y as isize + DY[dir];
        if out_field(nx, ny, self.h as isize, self.w as isize) {
            // フィールド外に出る場合は NG
            return false;
        }
        let nx = nx as usize;
        let ny = ny as usize;
        
        if grid_crane[nx][ny] != -1 {
            // 移動先にクレーンがいる場合は NG
            return false;
        }

        if !self.big && self.suspended && grid_cont[nx][ny][0] != -1 {
            // 小クレーンで吊り上げていて、移動先にコンテナがある場合は NG
            return false;
        }

        // if self.suspended && !self.big && ny == self.w - 1 {
        //     // 今すぐ搬出出来ないコンテナを持っている時に、右端に移動しようとした場合は NG
        //     let mut flag = false;
        //     for idx in out_cont_idx.iter().take(self.h) {
        //         flag |= *idx == grid_cont[self.x][self.y][self.big as usize] as usize;
        //     }
        //     if !flag {
        //         return false;
        //     }
        // }

        // クレーン 1,2 は左3列、クレーン 3,4 は右3列以外には移動できない
        // if (self.idx == 1 || self.idx == 2) && ny >= 3 {
        //     return false;
        // }
        true
    }

    fn suspend_ok(
        &mut self,
        grid_cont: &[Vec<Vec<i64>>],
        out_cont_idx: &[usize],
        appeard_cnt: usize,
        incoming_cont_idx: &[usize],
    ) -> bool {
        if self.suspended {
            // 吊り上げている場合は NG
            return false;
        }
        
        if grid_cont[self.x][self.y][0] == -1 {
            // 吊り上げるコンテナがない場合は NG
            return false;
        }

        // if self.idx != 0 {
        //     // 大クレーンではないのに、周囲が全てコンテナで囲まれている場所を吊り下げる場合は NG
        //     let mut flag = true;
        //     for i in 0..4 {
        //         let nx = self.x as isize + DX[i];
        //         let ny = self.y as isize + DY[i];
        //         if out_field(nx, ny, self.h as isize, self.w as isize) {
        //             continue;
        //         }
        //         let nx = nx as usize;
        //         let ny = ny as usize;
        //         flag &= grid_cont[nx][ny][0] != -1;
        //     }
        //     if flag {
        //         return false;
        //     }
        // }

        // if appeard_cnt >= 20 && self.y == 0 && incoming_cont_idx[self.x] != self.w {
        //     // 場に出てるコンテナが 15 個以上の場合は込み合うので NG
        //     return false;
        // }

        if self.idx == 0  {
            // crane 1 の場合は、今すぐ搬出可能なコンテナでない場合は NG
            let mut flag = false;
            for idx in out_cont_idx.iter().take(self.h) {
                flag |= *idx == grid_cont[self.x][self.y][0] as usize;
            }
            if !flag {
                return false;
            }
        }
        true
    }

    fn lower_ok(
        &mut self,
        grid_cont: &[Vec<Vec<i64>>],
        out_cont_idx: &[usize]
    ) -> bool {
        if !self.suspended {
            // 吊り上げていない場合は NG
            return false;
        }
        
        if self.big && grid_cont[self.x][self.y][0] != -1 {
            // 大クレーンで吊り下げ中で、降ろす場所にコンテナがある場合は NG
            return false;
        }

        if self.y == 1 {
            // 交通の便をよくするために、1 列目には降ろせない
            return false;
        }
        
        let cond_idx: usize = grid_cont[self.x][self.y][self.big as usize] as usize;
        let x = cond_idx / self.h;
        if !(self.y != self.w - 1 || self.x == x && out_cont_idx[x] == cond_idx) {
            // 降ろす場所が不適の場合は NG
            return false;
        }
        true
    }

    fn stop_ok(&mut self, grid_cont: &[Vec<Vec<i64>>]) -> bool {
        // stop はターン管理が難しいので false
        false
    }

    fn action_ok(
        &mut self,
        action: usize,
        grid_crane: &[Vec<isize>],
        grid_cont: &[Vec<Vec<i64>>],
        out_cont_idx: &[usize],
        incoming_cont_idx: &[usize],
        appeard_cnt: usize,
    ) -> bool {
        match action {
            0..=3 => self.shift_ok(action, grid_crane, grid_cont, out_cont_idx),
            4 => self.suspend_ok(grid_cont, out_cont_idx, appeard_cnt, incoming_cont_idx),
            5 => self.lower_ok(grid_cont, out_cont_idx),
            6 => self.stop_ok(grid_cont),
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
    out_cnt: usize,                      // 搬出済みのコンテナ数
    appeared_cnt: usize,                 // 場に出現したコンテナ数
    hash: u64,                           // Zobrist Hash
    conts: Vec<Vec<i64>>,                // 行 i から j 番目に来るコンテナの index
    out_cont_idx: Vec<usize>,            // 各搬出口から今搬出すべきコンテナの index
    incoming_cont_idx: Vec<usize>,       // 各搬入口から今搬入すべきコンテナの index
    grid_cont: Vec<Vec<Vec<i64>>>,       // (i, j) で上空(1) or 接地(0) が k のコンテナの index
    grid_crane: Vec<Vec<isize>>,         // (i, j) にいるクレーンの index
    cranes: Vec<Crane>,                  // 各クレーンの情報
    cont_suspended: Vec<Vec<Vec<bool>>>, // (i, j) にあるコンテナが吊り上げられているか
    
    out_cont_turn: Vec<Vec<(usize, i64)>>,   // 各搬出口から i ターン目に搬出したコンテナの (index, x)
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
            score: 1,
            turn: 0,
            out_cnt: 0,
            appeared_cnt: 0,
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
                self.appeared_cnt += 1;

                // そのターンに何を搬入したかを履歴として持つ
                self.incoming_cont_turn[self.turn].push(i);

                // 差分更新でスコア更新
                for j in self.incoming_cont_idx[i]..self.w {
                    let px = i as i64;
                    let gx = self.conts[i][j] / self.h as i64;
                    let perm = self.conts[i][j] % self.h as i64;
    
                    let npy1 = -(j as i64 - self.incoming_cont_idx[i] as i64 + 1);
                    let npy2 = npy1 + 1;
    
                    let mut sub = 0;
                    let mut add = 0;
    
                    // x 方向の寄与
                    sub += (px - gx).abs() * (px - gx).abs();
                    add += (px - gx).abs() * (px - gx).abs();
                    // y 方向の寄与
                    sub += (npy1 - (self.w - 1) as i64).abs() * (npy1 - (self.w - 1) as i64).abs();
                    add += (npy2 - (self.w - 1) as i64).abs() * (npy2 - (self.w - 1) as i64).abs();
                    // 倍率
                    sub *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
                    add *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
                    self.score += add - sub;
                }

                // 次に搬入すべきコンテナを更新
                self.incoming_cont_idx[i] += 1;
            }
        }
    }

    /* 今のターンで搬入口から搬入されたコンテナを元に戻す関数 */
    fn prepare_cont_revert(&mut self) {
        for i in self.incoming_cont_turn[self.turn].clone() {
            // 搬入したコンテナをクリア
            self.grid_cont[i][0][0] = -1;
            self.appeared_cnt -= 1;

            // 次に搬入すべきコンテナを戻す
            self.incoming_cont_idx[i] -= 1;

            // 差分更新でスコア更新
            for j in self.incoming_cont_idx[i]..self.w {
                let px = i as i64;
                let gx = self.conts[i][j] / self.h as i64;
                let perm = self.conts[i][j] % self.h as i64;

                let npy1 = -(j as i64 - self.incoming_cont_idx[i] as i64 + 1);
                let npy2 = npy1 + 1;

                let mut sub = 0;
                let mut add = 0;

                // x 方向の寄与
                sub += (px - gx).abs() * (px - gx).abs();
                add += (px - gx).abs() * (px - gx).abs();
                // y 方向の寄与
                sub += (npy2 - (self.w - 1) as i64).abs() * (npy2 - (self.w - 1) as i64).abs();
                add += (npy1 - (self.w - 1) as i64).abs() * (npy1 - (self.w - 1) as i64).abs();
                // 倍率
                sub *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
                add *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
                self.score += add - sub;
            }
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
                self.appeared_cnt -= 1;

                // 搬出したコンテナ数を更新
                self.out_cnt += 1;

                if self.out_cnt == self.h * self.w {
                    // 全てのコンテナを搬出した場合は終了
                    self.score = 0;
                    return;
                }
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
            self.appeared_cnt += 1;

            // 搬出したコンテナ数を戻す
            self.out_cnt -= 1;
        }
        self.out_cont_turn[self.turn].clear();
    }

    /* 次のノードに遷移する関数 */
    fn apply(&mut self, node: &Node) {
        let action = node.op;

        // 差分更新でスコア更新
        if action < 4 && self.cranes[CRANE_PERM[self.turn % (USING_CRANE)]].suspended {
            let crane = &self.cranes[CRANE_PERM[self.turn % (USING_CRANE)]];

            let px = crane.x as i64;
            let py = crane.y as i64;
            let nx = crane.x as i64 + DX[action] as i64;
            let ny = crane.y as i64 + DY[action] as i64;
            
            let cont = self.grid_cont[px as usize][py as usize][crane.big as usize];
            assert!(cont >= 0, "suspended cont is not found");
            let gx = cont / self.h as i64;
            let perm = cont % self.h as i64;

            let mut sub: i64 = 0;
            let mut add: i64 = 0;
            
            // x 方向の寄与
            sub += (px - gx).abs() * (px - gx).abs();
            add += (nx - gx).abs() * (nx - gx).abs();
            // y 方向の寄与
            sub += (py - (self.w - 1) as i64).abs() * (py - (self.w - 1) as i64).abs();
            add += (ny - (self.w - 1) as i64).abs() * (ny - (self.w - 1) as i64).abs();
            // 倍率
            sub *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
            add *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
            self.score += add - sub;
        }

        self.cranes[CRANE_PERM[self.turn % (USING_CRANE)]].action(
            action,
            &mut self.grid_crane,
            &mut self.grid_cont,
            &mut self.cont_suspended,
        );
        self.carry_out_cont();
        self.prepare_cont();
        self.turn += 1;
    }

    /* 前のノードに遷移する関数 */
    fn revert(&mut self, node: &Node) {
        let action = reverse_op(node.op) as usize;

        self.turn -= 1;
        self.prepare_cont_revert();
        self.carry_out_cont_revert();
        self.cranes[CRANE_PERM[self.turn % (USING_CRANE)]].action(
            action,
            &mut self.grid_crane,
            &mut self.grid_cont,
            &mut self.cont_suspended,
        );

        // 差分更新でスコア更新
        if action < 4 && self.cranes[CRANE_PERM[self.turn % (USING_CRANE)]].suspended {
            let crane = &self.cranes[CRANE_PERM[self.turn % (USING_CRANE)]];

            let px = crane.x as i64;
            let py = crane.y as i64;
            let nx = crane.x as i64 + DX[node.op] as i64;
            let ny = crane.y as i64 + DY[node.op] as i64;
            
            let cont = self.grid_cont[px as usize][py as usize][crane.big as usize];
            assert!(cont >= 0, "suspended cont is not found");
            let gx = cont / self.h as i64;
            let perm = cont % self.h as i64;

            let mut sub: i64 = 0;
            let mut add: i64 = 0;
            
            // x 方向の寄与
            sub += (nx - gx).abs() * (nx - gx).abs();
            add += (px - gx).abs() * (px - gx).abs();
            // y 方向の寄与
            sub += (ny - (self.w - 1) as i64).abs() * (ny - (self.w - 1) as i64).abs();
            add += (py - (self.w - 1) as i64).abs() * (py - (self.w - 1) as i64).abs();
            // 倍率
            sub *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
            add *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
            self.score += add - sub;
        }
    }

    fn evaluate(&self) -> i64 {
        /*
        ========== 評価関数 ==========
        目的地点とコンテナの二乗距離を d として、∑_{i,j} d(i,j) * 10^{何番目に搬出すべきか} の最小化を目指す
        */
        let mut score = 0;
        for i in 0..self.h {
            for j in 0..self.w {
                for k in 0..2 {
                    let gx = self.grid_cont[i][j][k] / self.h as i64;
                    let perm = self.grid_cont[i][j][k] % self.h as i64;
                    if self.grid_cont[i][j][k] != -1 {
                        let mut add = 0;
                        // x 方向の寄与
                        add += (i as i64 - gx).abs() * (i as i64 - gx).abs();
                        // y 方向の寄与
                        add += (j as i64 - (self.w - 1) as i64).abs() * (j as i64 - (self.w - 1) as i64).abs();
                        // 倍率
                        add *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
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
                add += (i as i64 - gx).abs() * (i as i64 - gx).abs();
                // y 方向の寄与
                add += ( -(j as i64 - self.incoming_cont_idx[i] as i64 + 1) - (self.w - 1) as i64).abs() * ( -(j as i64 - self.incoming_cont_idx[i] as i64 + 1) - (self.w - 1) as i64).abs();
                // 倍率
                add *= 10_i64.pow((self.w as i64 - perm) as u32 + 2);
                score += add;
            }
        }
        if self.out_cnt != self.h * self.w {
            score += 1;
        }
        score
    }
}

#[derive(Clone, Debug)]
struct Cand {
    op: usize,
    parent: usize,
    eval_score: i64,
}
impl Cand {
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
    /* [rhooさんの記事](https://qiita.com/rhoo/items/2f647e32f6ff2c6ee056)を参考 */
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

    // self.state が self.nodes[idx] のノードが表す状態になっている
    // self.nodes[idx] からの Cand を cands に積む
    fn append_cands(&mut self, idx: usize, cands: &mut Vec<Cand>) {
        let node = &self.nodes[idx];
        assert_eq!(node.child, !0);

        let mut next_exist = false;

        for _op in 0..OP_NUM {
            // 前回の逆操作は無視
            if reverse_op(_op) as usize == self.state.cranes[CRANE_PERM[self.state.turn % (USING_CRANE)]].pre_op {
                continue;
            }

            // 行動可能かを check
            if !self.state.cranes[CRANE_PERM[self.state.turn % (USING_CRANE)]].action_ok(
                _op,
                &self.state.grid_crane,
                &self.state.grid_cont,
                &self.state.out_cont_idx,
                &self.state.incoming_cont_idx,
                self.state.appeared_cnt,
            ) {
                continue;
            }

            // 盤面評価値を計算（差分計算で求める）
            let mut score = self.state.score;
            if _op < 4 && self.state.cranes[CRANE_PERM[self.state.turn % (USING_CRANE)]].suspended {
                let crane = &self.state.cranes[CRANE_PERM[self.state.turn % (USING_CRANE)]];

                let px = crane.x as i64;
                let py = crane.y as i64;
                let nx = crane.x as i64 + DX[_op] as i64;
                let ny = crane.y as i64 + DY[_op] as i64;
                
                let cont = self.state.grid_cont[px as usize][py as usize][crane.big as usize];
                // assert!(cont >= 0, "suspended cont is not found");
                let gx = cont / self.state.h as i64;
                let perm = cont % self.state.h as i64;

                let mut sub: i64 = 0;
                let mut add: i64 = 0;
                
                // x 方向の寄与
                sub += (px - gx).abs() * (px - gx).abs();
                add += (nx - gx).abs() * (nx - gx).abs();
                // y 方向の寄与
                sub += (py - (self.state.w - 1) as i64).abs() * (py - (self.state.w - 1) as i64).abs();
                add += (ny - (self.state.w - 1) as i64).abs() * (ny - (self.state.w - 1) as i64).abs();
                // 倍率
                sub *= 10_i64.pow((self.state.w as i64 - perm) as u32 + 2);
                add *= 10_i64.pow((self.state.w as i64 - perm) as u32 + 2);
                score += add - sub;

                // suspended && right && py == 0 の時は搬入口から出るコンテナによりそこの評価値変動も考慮
                if py == 0 && _op == 0 {
                    for i in self.state.incoming_cont_idx[px as usize]..self.state.w {
                        let gx = self.state.conts[px as usize][i] / self.state.h as i64;
                        let perm = self.state.conts[px as usize][i] % self.state.h as i64;

                        let npy1 = -(i as i64 - self.state.incoming_cont_idx[px as usize] as i64 + 1);
                        let npy2 = npy1 + 1;

                        add = 0;
                        sub = 0;
                        // x 方向の寄与
                        sub += (px - gx).abs() * (px - gx).abs();
                        add += (nx - gx).abs() * (nx - gx).abs();
                        // y 方向の寄与
                        sub += (npy1 - (self.state.w - 1) as i64).abs() * (npy1 - (self.state.w - 1) as i64).abs();
                        add += (npy2 - (self.state.w - 1) as i64).abs() * (npy2 - (self.state.w - 1) as i64).abs();
                        // 倍率
                        sub *= 10_i64.pow((self.state.w as i64 - perm) as u32 + 2);
                        add *= 10_i64.pow((self.state.w as i64 - perm) as u32 + 2);
                        
                        score += add - sub;
                    }
                }
                // assert!(score >= 0, "score is negative. score: {}", score);
            }

            next_exist = true;
            cands.push(
                Cand {
                    op: _op,
                    parent: idx,
                    eval_score: score,
                }
            );
        }

        if cands.len() < MAX_WIDTH && !next_exist {
            // 次の遷移が少なく遷移が無い場合は、仕方なく停止を考慮
            cands.push(
                Cand {
                    op: 6,
                    parent: idx,
                    eval_score: self.state.score,
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
    let initial_node: Node = {
        Node {
            op: !0,
            parent: !0,
            child: !0,
            prev: !0,
            next: !0,
        }
    };

    // let mut actions: String = "PPPPPRRRRRRRRRRQQQQQ".to_string();
    let mut actions: String = "".to_string();
    
    let mut initial_terminal = Terminal::new(&input);
    for (i, action) in actions.chars().enumerate() {
        initial_terminal.prepare_cont();
        initial_terminal.cranes[i % 5].action(
            OP.iter().position(|&x| x == action).unwrap(),
            &mut initial_terminal.grid_crane,
            &mut initial_terminal.grid_cont,
            &mut initial_terminal.cont_suspended,
        );
    }
    // USING_CRANE 以外は爆破
    // for i in USING_CRANE..5 {
    //     initial_terminal.cranes[i].action(
    //         7,
    //         &mut initial_terminal.grid_crane,
    //         &mut initial_terminal.grid_cont,
    //         &mut initial_terminal.cont_suspended,
    //     );
    // }
    initial_terminal.prepare_cont();
    initial_terminal.incoming_cont_turn[0].clear();
    initial_terminal.score = initial_terminal.evaluate();
    let mut solver = BeamSearch::new(
        initial_terminal,
        initial_node,
    );
    eprintln!("initial score: {}", solver.state.score);

    let mut best_idx: usize = !0;

    for turn in 0..TURN {
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
        assert!(!_top_cands.is_empty());
        if _top_cands[0].eval_score == 0 {
            best_idx = _top_cands[0].parent;
            break;
        }
        
        // 候補を基に次の状態を更新
        solver.update(top_cands);
    }

    // 最終状態の復元
    assert!(best_idx != !0);
    let final_path = solver.restore(best_idx);
    for op in final_path {
        actions.push(OP[op]);
    }

    write_output(actions)
}
