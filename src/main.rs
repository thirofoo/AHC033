use proconio::*;
use std::collections::VecDeque;

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
    grid_crane: Vec<Vec<i8>>,
    grid_container: Vec<Vec<Vec<i8>>>,
    container_idx: Vec<u8>,
    next_container: Vec<u8>,
}

impl Terminal {
    fn new(input: &Input) -> Self {
        let mut _next_container: Vec<u8> = vec![0; input.n];
        for (i, crane) in _next_container.iter_mut().enumerate() {
            *crane = i as u8 * input.n as u8;
        }
        Self {
            h: input.n,
            w: input.n,
            grid_crane: vec![vec![-1; input.n]; input.n],
            grid_container: vec![vec![vec![-1; 2]; input.n]; input.n],
            container_idx: vec![0; input.n],
            next_container: _next_container,
        }
    }

    fn prepare_container(&mut self, input: &Input) {
        /* 盤面の左端・右端のコンテナを展開・搬出する関数 */
        for i in 0..self.h {
            /* 展開 */
            if self.grid_container[i][0][0] == -1 && self.container_idx[i] < input.n as u8 {
                self.grid_container[i][0][0] = input.a[i][self.container_idx[i] as usize] as i8;
                self.container_idx[i] += 1;
            }
            /* 搬出 */
            if self.grid_container[i][self.w-1][0] != -1 {
                let container_idx = self.grid_container[i][self.w-1][0];
                self.grid_container[i][self.w-1][0] = -1;
                self.next_container[(container_idx / self.h as i8) as usize] = container_idx as u8 + 1;
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
    fn new(input: &Input, _idx: usize, _x: usize, _y: usize, _big: bool, terminal: &mut Terminal) -> Self {
        assert!(terminal.grid_crane[_x][_y] == -1);
        terminal.grid_crane[_x][_y] = _idx as i8;
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

    fn shift(&mut self, dir: usize, terminal: &mut Terminal, input: &Input) -> char {
        assert!(!self.exploded);
        let nx = self.x as isize + DX[dir];
        let ny = self.y as isize + DY[dir];

        /* 場外に出ていないかを確認 */
        assert!(!out_field(nx, ny, self.h as isize, self.w as isize));
        let nx = nx as usize;
        let ny = ny as usize;

        /* 隣にクレーンが無いことを確認 */
        assert!(terminal.grid_crane[nx][ny] == -1);

        /* suspend している場合に移動先にコンテナが無いかを確認 */
        if self.suspended {
            assert!(terminal.grid_container[nx][ny][self.big as usize] == -1);
        }

        terminal.grid_crane[nx][ny] = self.idx as i8;
        terminal.grid_crane[self.x][self.y] = -1;

        if self.suspended {
            let container_idx = terminal.grid_container[self.x][self.y][self.big as usize];
            terminal.grid_container[nx][ny][self.big as usize] = container_idx;
            terminal.grid_container[self.x][self.y][self.big as usize] = -1;
        }

        if self.x == 0 {
            terminal.prepare_container(input);
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
            let container_idx = terminal.grid_container[self.x][self.y][0];
            terminal.grid_container[self.x][self.y][1] = container_idx;
            terminal.grid_container[self.x][self.y][0] = -1;
        }
        'P'
    }

    fn lower(&mut self, terminal: &mut Terminal) -> char {
        assert!(!self.exploded);
        assert!(self.suspended);
        self.suspended = false;
        if self.big {
            let container_idx = terminal.grid_container[self.x][self.y][1];
            terminal.grid_container[self.x][self.y][0] = container_idx;
            terminal.grid_container[self.x][self.y][1] = -1;
        }
        'Q'
    }

    fn stop(&self) -> char {
        '.'
    }

    fn explode(&mut self, terminal: &mut Terminal) -> char {
        assert!(!self.exploded);
        self.exploded = true;
        terminal.grid_crane[self.x][self.y] = -1;
        terminal.grid_crane[self.x][self.y] = -1;
        'X'
    }

    fn move_crane(
        &mut self,
        goal: (usize, usize),
        terminal: &mut Terminal,
        input: &Input
    ) -> String {
        /* BFS と経路復元で Crane を目的地まで動かす関数 */
        let mut res: String = "".to_string();

        /* queue を用意 */
        let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();
        queue.push_back((self.x, self.y, 0));
        let mut dis: Vec<Vec<usize>> = vec![vec![std::usize::MAX; terminal.w]; terminal.h];

        while !queue.is_empty() {
            let (x, y, d) = queue.pop_front().unwrap();
            if (x, y) == goal {
                dis[x][y] = d;
                break;
            }
            if dis[x][y] != std::usize::MAX {
                continue;
            }
            dis[x][y] = d;
            for dir in 0..DIR_NUM {
                let nx = x as isize + DX[dir];
                let ny = y as isize + DY[dir];
                if out_field(nx, ny, terminal.h as isize, terminal.w as isize) {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if terminal.grid_crane[nx][ny] != -1 || dis[nx][ny] != std::usize::MAX {
                    continue;
                }
                /* small の時はコンテナがある場所には移動できない */
                if !self.big && terminal.grid_container[nx][ny][0] != -1 {
                    continue;
                }
                queue.push_back((nx, ny, d + 1));
            }
        }
        /* 経路復元 */
        eprintln!("dis: {:?}", dis);
        let mut x = goal.0;
        let mut y = goal.1;
        let mut d = dis[x][y];
        let mut dirs: Vec<usize> = vec![];
        while d > 0 {
            for dir in 0..DIR_NUM {
                let nx = x as isize + DX[dir];
                let ny = y as isize + DY[dir];
                if out_field(nx, ny, terminal.h as isize, terminal.w as isize) {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if dis[nx][ny] == d - 1 {
                    dirs.push((dir+2)%DIR_NUM);
                    x = nx;
                    y = ny;
                    d -= 1;
                    break;
                }
            }
        }
        dirs = dirs.into_iter().rev().collect();
        for dir in dirs {
            res.push(self.shift(dir, terminal, input));
        }
        res
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

    fn balance(&mut self) {
        /* 一番手数が多いクレーンに合わせて stop して均す関数 */
        let max_len = self.log.iter().map(|x| x.len()).max().unwrap();
        for i in 0..self.log.len() {
            while self.log[i].len() < max_len {
                self.log[i].push('.');
            }
        }
    }
}

fn main() {
    /*
    ========== 貪欲解法 ==========
    1. 20 マス分コンテナを展開する (残り 5 個が 0, 5, 10, 15, 20 のケースは一旦考えない)
    2. 小クレーンを右端に移動
    3. 大クレーンで今現在正しく運び出せるコンテナを右端に移動
      - この時、小クレーンが邪魔になる場合は、右端の上下で上手く移動させる
    4. 大クレーンで右端に詰める
    5. 各搬出口の最後以外のコンテナが積み終わるまで 3, 4 を繰り返す
    */

    let input = Input::read_input();
    let mut terminal = Terminal::new(&input);
    terminal.prepare_container(&input);

    let mut cranes: Vec<Crane> = vec![];
    for i in 0..input.n {
        let big = i == 0;
        cranes.push(Crane::new(&input, i, i, 0, big, &mut terminal));
    }
    let mut actions = Action::new(input.n);

    /* 1. 20 マス分コンテナを展開する */
    for y in (1..input.n-1).rev() {
        for (i, crane) in cranes.iter_mut().enumerate() {
            actions.push(i, crane.suspend(&mut terminal));
            for _ in 0..y {
                actions.push(i, crane.shift(Direction::Right as usize, &mut terminal, &input));
            }
            actions.push(i, crane.lower(&mut terminal));
            for _ in 0..y {
                actions.push(i, crane.shift(Direction::Left as usize, &mut terminal, &input));
            }
        }
    }

    /* 2. 小クレーンを右端に移動 */
    for (i, crane) in cranes.iter_mut().enumerate() {
        while crane.y < input.n - 1 {
            actions.push(i, crane.shift(Direction::Right as usize, &mut terminal, &input));
        }
    }

    let mut carried = 0;
    loop {
        loop {
            /* 3. 大クレーンで今現在正しく運び出せるコンテナを右端に移動 */
            let mut point: (i32, i32) = (-1, -1);
            let mut goal: (i32, i32) = (-1, -1);
            for i in 0..input.n {
                for j in 0..input.n {
                    for (k, container) in terminal.next_container.iter().enumerate() {
                        if *container >= (k+1) as u8 * input.n as u8 {
                            continue;
                        }
                        if terminal.grid_container[i][j][0] == *container as i8 {
                            point = (i as i32, j as i32);
                            goal = (k as i32, input.n as i32 - 1);
                        }
                    }
                }
            }
            if point == (-1, -1) {
                break;
            }
            let point = (point.0 as usize, point.1 as usize);
            let goal = (goal.0 as usize, goal.1 as usize);
            eprintln!("point: {:?}, goal: {:?}", point, goal);
            
            /* 目的コンテナまで移動 */
            let trace: String = cranes[0].move_crane(point, &mut terminal, &input);
            for step in trace.chars() {
                actions.push(0, step);
            }
            actions.push(0, cranes[0].suspend(&mut terminal));
            actions.balance();

            eprintln!("terminal.grid_container: {:?}", terminal.grid_container);
            eprintln!("terminal.grid_crane: {:?}", terminal.grid_crane);

            /* 搬出口手前にクレーンがある場合にどける */
            let mut empty_point = (0, 0);
            for i in 0..input.n {
                if terminal.grid_crane[i][input.n - 1] == -1 {
                    empty_point = (i, input.n - 1);
                    break;
                }
            }
            eprintln!("empty_point: {:?}", empty_point);

            match goal.0.cmp(&empty_point.0) {
                std::cmp::Ordering::Less => {
                    for i in (goal.0+1..empty_point.0+1).rev() {
                        actions.push(i, cranes[i].shift(Direction::Down as usize, &mut terminal, &input));
                    }
                },
                std::cmp::Ordering::Greater => {
                    for i in empty_point.0+1..goal.0+1 {
                        actions.push(i, cranes[i].shift(Direction::Up as usize, &mut terminal, &input));
                    }
                },
                _ => {}
            }
            actions.balance();

            eprintln!("terminal.grid_container: {:?}", terminal.grid_container);
            eprintln!("terminal.grid_crane: {:?}", terminal.grid_crane);

            /* 搬出口まで移動 */
            let trace: String = cranes[0].move_crane(goal, &mut terminal, &input);
            for step in trace.chars() {
                actions.push(0, step);
            }
            actions.push(0, cranes[0].lower(&mut terminal));
            actions.balance();
            carried += 1;
            terminal.prepare_container(&input);

            eprintln!("terminal.grid_container: {:?}", terminal.grid_container);
            eprintln!("terminal.grid_crane: {:?}", terminal.grid_crane);
            eprintln!("carried: {}\n\n", carried);
        }

        /* 4. 大クレーンで右端に詰める */
        for i in 0..input.n {
            for j in (0..input.n-2).rev() {
                if terminal.grid_container[i][j][0] != -1 && terminal.grid_container[i][j+1][0] == -1 {
                    let point = (i, j);
                    let goal = (i, j+1);
                    let trace: String = cranes[0].move_crane(point, &mut terminal, &input);
                    for step in trace.chars() {
                        actions.push(0, step);
                    }
                    actions.push(0, cranes[0].suspend(&mut terminal));
                    let trace: String = cranes[0].move_crane(goal, &mut terminal, &input);
                    for step in trace.chars() {
                        actions.push(0, step);
                    }
                    actions.push(0, cranes[0].lower(&mut terminal));
                    actions.balance();
                }
            }
        }
        terminal.prepare_container(&input);

        eprintln!("carried: {}", carried);
        if carried == input.n * input.n {
            /* 5. 各搬出口の最後以外のコンテナが積み終わるまで 3 ~ 6 を繰り返す */
            break;
        }
    }    

    write_output(actions);
}