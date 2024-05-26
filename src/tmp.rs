#![allow(non_snake_case)]


fn main(){
    let _=std::thread::Builder::new().name("run".to_string()).stack_size(32*1024*1024).spawn(run).unwrap().join();
}


fn run(){
    get_time();
    let input=Input::input();
    let ans=solve(&input);

    println!("{}",ans.len());

    for &n in &ans{
        let (i0,j0)=input.to_pos[n.0];
        let (i1,j1)=input.to_pos[n.1];
        println!("{} {} {} {}",i0,j0,i1,j1);
    }
}


fn solve(input:&Input)->Vec<(usize,usize)>{
    let state=State::new(input);
    let node={
        let mut n=0;
        while state.isok(input,n){
            n+=1;
        }
        let score=state.score(input,n);
        let hash=state.hash(input,n);
        Node{
            op:[(!0,!0);2],
            parent:!0,
            child:!0,
            prev:!0,
            next:!0,
            score,n,hash,
            prev_pos:!0,
            refs:0,
            valid:0,
        }
    };

    let mut solver=BeamSearch::new(state.clone(),node.clone());
    
    let mut best=vec![];
    let mut best_score=!0;
    let mut iter=0;
    loop{
        iter+=1;
        solver.reset(state.clone(),node.clone());
        let res=solver.solve(input);
        if res.is_empty(){
            break;
        }
        if best_score>res.len(){
            best_score=res.len();
            best=res;
        }
    }

    eprintln!("iter = {}",iter);
    eprintln!("score = {}",best_score);
    eprintln!("time = {}",get_time());

    best
}


const ENDED_BONUS:i64=1200;
static mut RR:usize=30;


#[allow(non_camel_case_types)]
type uint=u32;


#[derive(Clone)]
struct State{
    state:[usize;L+1],
    pos:[usize;L],
}
impl State{
    fn new(input:&Input)->State{
        let mut pos=[!0;L];
        for (i,&n) in input.grid.iter().enumerate(){
            pos[n]=i;
        }
        let mut state=[0;L+1];
        state[..L].copy_from_slice(&input.grid);

        State{state,pos}
    }

    fn isok(&self,input:&Input,n:usize)->bool{
        if n==465{
            return false;
        }
        let p=self.pos[n];
        let np0=input.dd[p][0];
        let np1=input.dd[p][1];
        
        n>=self.state[np0] && n>=self.state[np1]
    }

    fn score(&self,input:&Input,n:usize)->i64{
        (0..L).map(|i|
            input.height[i]*self.state[i] as i64
        ).sum::<i64>()
        +n as i64*ENDED_BONUS
    }

    fn hash(&self,input:&Input,n:usize)->u64{
        let mut hash=0;
        for i in 0..n{
            hash^=input.zob[self.pos[i]];
        }
        hash
    }

    fn apply(&mut self,node:&Node){
        self.swap(node.op[0]);
        self.swap(node.op[1]);
    }

    fn revert(&mut self,node:&Node){
        self.swap(node.op[1]);
        self.swap(node.op[0]);
    }

    fn swap(&mut self,(p0,p1):(usize,usize)){
        if p0==L{
            return;
        }
        self.pos.swap(self.state[p0],self.state[p1]);
        self.state.swap(p0,p1);
    }
}


#[derive(Clone)]
struct Cand{
    op:[(usize,usize);2],
    parent:uint,
    eval_score:i64,
    hash:u64,
    n:usize,
    score:i64,
    prev_pos:usize,
    pos:usize,
}
impl Cand{
    fn raw_score(&self,input:&Input)->i64{
        self.score
    }
    
    fn to_node(&self)->Node{
        Node{
            child:!0,
            prev:!0,
            next:!0,
            op:self.op,
            parent:self.parent,
            score:self.score,
            n:self.n,
            hash:self.hash,
            prev_pos:self.prev_pos,
            valid:0,
            refs:0,
        }
    }
}


#[derive(Clone,Default)]
struct Node{
    op:[(usize,usize);2],
    parent:uint,
    child:uint,
    prev:uint,
    next:uint,
    score:i64,
    n:usize,
    hash:u64,
    prev_pos:usize,
    refs:u8,
    valid:u16,
}


const MAX_WIDTH:usize=300;
// const TURN:usize=100;


#[derive(Clone)]
struct BeamSearch{
    state:State,
    nodes:Vec<Node>,
    que:Vec<uint>,
    cur_node:usize,
    free:Vec<uint>,
    at:u16,
}
impl BeamSearch{
    fn new(state:State,node:Node)->BeamSearch{
        const MAX_NODES:usize=MAX_WIDTH*10;
        assert!(MAX_NODES<uint::MAX as usize,"uintのサイズが足りないよ");
        let mut nodes=vec![Node::default();MAX_NODES];
        nodes[0]=node;
        let free=(1..MAX_NODES as uint).rev().collect();

        BeamSearch{
            state,nodes,free,
            que:Vec::with_capacity(MAX_WIDTH),
            cur_node:0,
            at:0,
        }
    }
    
    fn reset(&mut self,state:State,mut node:Node){
        self.state=state;
        self.at+=1;
        node.valid=self.at;
        self.nodes[0]=node;
        self.free.clear();
        self.free.extend((1..self.nodes.len() as uint).rev());
        self.que.clear();
        self.cur_node=0;
    }
    
    fn add_node(&mut self,cand:Cand){
        let next=self.nodes[cand.parent as usize].child;

        let new=self.free.pop().unwrap_or_else(||{
            let n=self.nodes.len() as uint;
            assert!(n!=0,"uintのサイズが足りないよ");
            self.nodes.push(Node::default());
            n
        });

        if next!=!0{
            self.nodes[next as usize].prev=new;
        }
        self.nodes[cand.parent as usize].child=new;
        
        self.nodes[new as usize]=Node{next,..cand.to_node()};
        self.retarget(new);
    }

    fn del_node(&mut self,mut idx:uint){
        assert!(self.nodes[idx as usize].refs==0);
        
        loop{
            self.free.push(idx);
            let Node{prev,next,parent,..}=self.nodes[idx as usize];
            assert_ne!(parent,!0,"全てのノードを消そうとしています");

            self.nodes[parent as usize].refs-=1;

            if prev&next==!0 && self.nodes[parent as usize].refs==0{
                idx=parent;
                continue;
            }

            if prev!=!0{
                self.nodes[prev as usize].next=next;
            }
            else{
                self.nodes[parent as usize].child=next;
            }
            if next!=!0{
                self.nodes[next as usize].prev=prev;
            }
            
            break;
        }
    }

    fn dfs(&mut self,input:&Input,turn:usize,cands:&mut Vec<Vec<Cand>>,single:bool){
        if self.nodes[self.cur_node].child==!0{
            let cnt=self.append_cands(input,turn,self.cur_node,cands);
            if cnt==0{
                self.que.push(self.cur_node as uint);
            }
            self.nodes[self.cur_node].refs+=cnt;
            return;
        }

        let node=self.cur_node;
        let mut child=self.nodes[node].child;
        let next_single=single&(self.nodes[child as usize].next==!0);

        // let prev_state=self.state.clone();
        'a: loop{
            while self.nodes[child as usize].valid!=self.at{
                child=self.nodes[child as usize].next;
                if child==!0{
                    break 'a;
                }
            }
            
            self.cur_node=child as usize;
            self.state.apply(&self.nodes[child as usize]);
            self.dfs(input,turn,cands,next_single);

            if !next_single{
                self.state.revert(&self.nodes[child as usize]);
                // assert!(prev_state==self.state);
            }

            child=self.nodes[child as usize].next;
            if child==!0{
                break;
            }
        }

        if !next_single{
            self.cur_node=node;
        }
    }

    fn no_dfs(&mut self,input:&Input,turn:usize,cands:&mut Vec<Vec<Cand>>){
        loop{
            let Node{next,child,..}=self.nodes[self.cur_node];
            if next==!0 || child==!0{
                break;
            }
            self.cur_node=child as usize;
            self.state.apply(&self.nodes[self.cur_node]);
        }

        assert!(self.nodes[self.cur_node].valid==self.at);
        let root=self.cur_node;

        loop{
            assert!(self.nodes[self.cur_node].valid==self.at);
            let mut child=self.nodes[self.cur_node].child;

            if child==!0{
                let cnt=self.append_cands(input,turn,self.cur_node,cands);
                if cnt==0{
                    self.que.push(self.cur_node as uint);
                }
                self.nodes[self.cur_node].refs+=cnt;

                'a: loop{
                    if self.cur_node==root{
                        return;
                    }
                    let node=&self.nodes[self.cur_node];
                    self.state.revert(&node);
                    let mut next=node.next;
                    loop{
                        if next==!0{
                            self.cur_node=node.parent as usize;
                            break;
                        }
                        if self.nodes[next as usize].valid==self.at{
                            self.cur_node=next as usize;
                            self.state.apply(&self.nodes[self.cur_node]);
                            break 'a;
                        }
                        next=self.nodes[next as usize].next;
                    }
                }
            }
            else{
                while self.nodes[child as usize].valid!=self.at{
                    child=self.nodes[child as usize].next;
                }
                self.cur_node=child as usize;
                self.state.apply(&self.nodes[self.cur_node]);
            }
        }
    }

    fn enum_cands(&mut self,input:&Input,turn:usize,cands:&mut Vec<Vec<Cand>>){
        assert_eq!(self.nodes[self.cur_node].valid,self.at);
        self.que.clear();
        // self.dfs(input,turn,cands,true);
        self.no_dfs(input,turn,cands);
    }
    
    fn retarget(&mut self,mut idx:uint){
        while self.nodes[idx as usize].valid!=self.at{
            self.nodes[idx as usize].valid=self.at;
            if idx as usize==self.cur_node{
                break;
            }
            idx=self.nodes[idx as usize].parent;
        }
    }

    fn update<I:Iterator<Item=(Cand,bool)>>(&mut self,cands:I){
        self.at+=1;
        for i in 0..self.que.len(){
            self.del_node(self.que[i]);
        }
        
        for (cand,f) in cands{
            let node=&mut self.nodes[cand.parent as usize];
            if f{
                self.add_node(cand);
            } else{
                node.refs-=1;
                if node.refs==0{
                    self.del_node(cand.parent);
                }
            }
        }
    }

    fn restore(&self,mut idx:uint)->Vec<(usize,usize)>{
        let mut ret=vec![];
        loop{
            let Node{op,parent,..}=self.nodes[idx as usize];
            if parent==!0{
                break;
            }
            if op[1].0!=L{
                ret.push(op[1]);
            }
            ret.push(op[0]);
            idx=parent;
        }
        
        ret.reverse();
        ret
    }

    fn append_cands(&mut self,input:&Input,turn:usize,idx:usize,cands:&mut Vec<Vec<Cand>>)->u8{
        let node=&self.nodes[idx];
        assert_eq!(node.child,!0);
        // assert_eq!(node.score,self.state.score(input,node.n));
        // assert_eq!(node.hash,self.state.hash(input,node.n));

        let pos=self.state.pos[node.n];
        let mut ret=0;

        for &i in &[0,1,2,5]{
            let np=input.dd[pos][i];
            if node.n>=self.state.state[np] || np==node.prev_pos{
                continue;
            }

            let mut score=node.score;
            self.state.swap((pos,np));
            
            let mut n=node.n;
            let mut hash=node.hash;
            
            while self.state.isok(input,n){
                let pos=self.state.pos[n];
                hash^=input.zob[pos];
                n+=1;
            }

            let tp=*self.state.pos.get(n).unwrap_or(&!0);
            let prev_pos=if n!=node.n{
                !0
            } else {
                pos
            };

            score+=ENDED_BONUS*(n-node.n) as i64;
            self.state.swap((pos,np));

            let nn=self.state.state[np];
            let old=input.height[pos]*node.n as i64+input.height[np]*nn as i64;
            let new=input.height[pos]*nn as i64+input.height[np]*node.n as i64;
            score+=new-old;

            let cand=Cand{
                op:[(pos,np),(L,L)],
                parent:idx as uint,
                score,n,hash,
                eval_score:score+(rnd::next()%unsafe{RR}) as i64,
                prev_pos,
                pos:tp,
            };
            cands[turn+1].push(cand);
            ret+=1;
        }

        for &(np,op) in &input.dd2[pos]{
            if node.n>=self.state.state[np] || np==node.prev_pos{
                continue;
            }

            let mut score=node.score;
            self.state.swap(op[0]);
            self.state.swap(op[1]);

            let mut n=node.n;
            let mut hash=node.hash;
            
            while self.state.isok(input,n){
                let pos=self.state.pos[n];
                hash^=input.zob[pos];
                n+=1;
            }

            let tp=*self.state.pos.get(n).unwrap_or(&!0);
            let prev_pos=if n!=node.n{
                !0
            } else {
                pos
            };

            score+=ENDED_BONUS*(n-node.n) as i64;
            self.state.swap(op[1]);
            self.state.swap(op[0]);

            let nn=self.state.state[np];
            let mm=self.state.state[op[0].1];
            let old=input.height[pos]*node.n as i64+input.height[op[0].1]*mm as i64+input.height[np]*nn as i64;
            let new=input.height[pos]*nn as i64+input.height[op[0].1]*node.n as i64+input.height[np]*mm as i64;
            score+=new-old;

            let cand=Cand{
                op,
                parent:idx as uint,
                score,n,hash,
                eval_score:score+(rnd::next()%unsafe{RR}) as i64,
                prev_pos,
                pos:tp,
            };
            cands[turn+2].push(cand);
            ret+=1;
        }

        ret
    }

    fn solve(&mut self,input:&Input)->Vec<(usize,usize)>{
        use std::cmp::Reverse;
        let M=MAX_WIDTH;
    
        let mut cands=(0..=3000).map(|_|Vec::<Cand>::with_capacity(MAX_WIDTH*8)).collect::<Vec<_>>();
        let mut first=true;
        let mut set=NopHashSet::default();
        let best;
        let mut turn=0;
        
        loop{
            const R0:f64=40.;
            const R1:f64=20.;
            unsafe{
                RR=(R0+(R1-R0)*(turn as f64/1900.)).round() as usize;
            }
            
            if !first{
                let cands=&mut cands[turn];
                assert!(!cands.is_empty());
                if let Some(idx)=(0..cands.len()).find(|&i|cands[i].n==L){
                    best=cands[idx].clone();
                    break;
                }
                
                let M0=(M as f64*2.).round() as usize;
                if cands.len()>M0{
                    cands.select_nth_unstable_by_key(M0,|a|Reverse(a.score));
                }
                let len=M0.min(cands.len());
                cands[..len].sort_unstable_by_key(|a|Reverse(a.eval_score));

                set.clear();
                let mut total=0;
                self.update(cands.drain(..).map(|cand|{
                    let f=total<M && set.insert(cand.hash^input.zob[cand.pos]);
                    total+=f as usize;
                    (cand,f)
                }));
            }
            first=false;

            self.enum_cands(input,turn,&mut cands);

            turn+=1;
            if turn&31==0 && get_time()>=1.95{
                return vec![];
            }
        }

        let mut ret=self.restore(best.parent);
        ret.push(best.op[0]);
        if best.op[1].0!=L{
            ret.push(best.op[1]);
        }

        ret
    }
}


const N:usize=30;
const L:usize=465;


use proconio::*;
use rand::prelude::*;


struct Input{
    grid:[usize;L],
    zob:[u64;L],
    dd:[[usize;6];L],
    dd2:[[(usize,[(usize,usize);2]);4];L],
    height:[i64;L],
    to_pos:[(usize,usize);L],
}
impl Input{
    fn input()->Input{
        let mut to_id=[[!0;N];N];
        let mut it=0..;
        let mut to_pos=[(!0,!0);L];
        for i in 0..N{
            for j in 0..=i{
                to_id[i][j]=it.next().unwrap();
                to_pos[to_id[i][j]]=(i,j);
            }
        }
        
        input!{
            i_grid:[usize;L]
        }
        let mut grid=[!0;L];
        grid.copy_from_slice(&i_grid);

        let mut zob=[!0;L];
        let mut rng=rand_pcg::Pcg64Mcg::new(0);
        for i in 0..L{
            zob[i]=rng.gen();
        }

        let mut dd=[[L;6];L];
        for &n in to_pos.iter(){
            for (i,&d) in [(!0,!0),(!0,0),(0,1),(1,1),(1,0),(0,!0)].iter().enumerate(){
                let np=(n.0+d.0,n.1+d.1);
                if np.0<N && np.1<N && to_id[np.0][np.1]!=!0{
                    dd[to_id[n.0][n.1]][i]=to_id[np.0][np.1];
                }
            }
        }

        let mut height=[0;L];
        for i in 0..L{
            height[i]=(to_pos[i].0 as f64).powf(1.).round() as i64; // todo
        }

        let mut dd2=[[(L,[(L,L);2]);4];L];
        for i in 0..L{
            for j in 0..4{
                let mp=dd[i][j&1];
                if mp==L{
                    continue;
                }
                let np=dd[mp][j>>1];
                dd2[i][j]=(np,[(np,mp),(mp,i)])
            }
        }

        Input{grid,zob,dd,dd2,height,to_pos}
    }
}



fn get_time()->f64{
    static mut START:f64=-1.;
    let time=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
    unsafe{
        if START<0.{
            START=time;
        }

        #[cfg(local)]{
            (time-START)*1.5
        }
        #[cfg(not(local))]{
            time-START
        }
    }
}


#[allow(unused)]
mod nth{
    use std::cmp;
    use std::mem::{self, MaybeUninit};
    use std::ptr;
    
    struct CopyOnDrop<T> {
        src: *const T,
        dest: *mut T,
    }
    
    impl<T> Drop for CopyOnDrop<T> {
        fn drop(&mut self) {
            unsafe {
                ptr::copy_nonoverlapping(self.src, self.dest, 1);
            }
        }
    }
    
    fn shift_tail<T, F>(v: &mut [T], is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let len = v.len();
        unsafe {
            if len >= 2 && is_less(v.get_unchecked(len - 1), v.get_unchecked(len - 2)) {
                let tmp = mem::ManuallyDrop::new(ptr::read(v.get_unchecked(len - 1)));
                let v = v.as_mut_ptr();
                let mut hole = CopyOnDrop { src: &*tmp, dest: v.add(len - 2) };
                ptr::copy_nonoverlapping(v.add(len - 2), v.add(len - 1), 1);
    
                for i in (0..len - 2).rev() {
                    if !is_less(&*tmp, &*v.add(i)) {
                        break;
                    }
    
                    ptr::copy_nonoverlapping(v.add(i), v.add(i + 1), 1);
                    hole.dest = v.add(i);
                }
            }
        }
    }
    
    fn insertion_sort<T, F>(v: &mut [T], is_less: &mut F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        for i in 1..v.len() {
            shift_tail(&mut v[..i + 1], is_less);
        }
    }
    
    fn partition_in_blocks<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        const BLOCK: usize = 128;
    
        let mut l = v.as_mut_ptr();
        let mut block_l = BLOCK;
        let mut start_l = ptr::null_mut();
        let mut end_l = ptr::null_mut();
        let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];
    
        let mut r = unsafe { l.add(v.len()) };
        let mut block_r = BLOCK;
        let mut start_r = ptr::null_mut();
        let mut end_r = ptr::null_mut();
        let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];
    
        fn width<T>(l: *mut T, r: *mut T) -> usize {
            assert!(mem::size_of::<T>() > 0);
            (r as usize - l as usize) / mem::size_of::<T>()
        }
    
        loop {
            let is_done = width(l, r) <= 2 * BLOCK;
    
            if is_done {
                let mut rem = width(l, r);
                if start_l < end_l || start_r < end_r {
                    rem -= BLOCK;
                }
    
                if start_l < end_l {
                    block_r = rem;
                } else if start_r < end_r {
                    block_l = rem;
                } else {
                    block_l = rem / 2;
                    block_r = rem - block_l;
                }
                debug_assert!(block_l <= BLOCK && block_r <= BLOCK);
                debug_assert!(width(l, r) == block_l + block_r);
            }
    
            if start_l == end_l {
                start_l = offsets_l.as_mut_ptr() as *mut _;
                end_l = start_l;
                let mut elem = l;
    
                for i in 0..block_l {
                    unsafe {
                        *end_l = i as u8;
                        end_l = end_l.offset(!is_less(&*elem, pivot) as isize);
                        elem = elem.offset(1);
                    }
                }
            }
    
            if start_r == end_r {
                start_r = offsets_r.as_mut_ptr() as *mut _;
                end_r = start_r;
                let mut elem = r;
    
                for i in 0..block_r {
                    unsafe {
                        elem = elem.offset(-1);
                        *end_r = i as u8;
                        end_r = end_r.offset(is_less(&*elem, pivot) as isize);
                    }
                }
            }
    
            let count = cmp::min(width(start_l, end_l), width(start_r, end_r));
    
            if count > 0 {
                macro_rules! left {
                    () => {
                        l.offset(*start_l as isize)
                    };
                }
                macro_rules! right {
                    () => {
                        r.offset(-(*start_r as isize) - 1)
                    };
                }
    
                unsafe {
                    let tmp = ptr::read(left!());
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
    
                    for _ in 1..count {
                        start_l = start_l.offset(1);
                        ptr::copy_nonoverlapping(left!(), right!(), 1);
                        start_r = start_r.offset(1);
                        ptr::copy_nonoverlapping(right!(), left!(), 1);
                    }
    
                    ptr::copy_nonoverlapping(&tmp, right!(), 1);
                    mem::forget(tmp);
                    start_l = start_l.offset(1);
                    start_r = start_r.offset(1);
                }
            }
    
            if start_l == end_l {
                l = unsafe { l.offset(block_l as isize) };
            }
    
            if start_r == end_r {
                r = unsafe { r.offset(-(block_r as isize)) };
            }
    
            if is_done {
                break;
            }
        }
    
        if start_l < end_l {
            debug_assert_eq!(width(l, r), block_l);
            while start_l < end_l {
                unsafe {
                    end_l = end_l.offset(-1);
                    ptr::swap(l.offset(*end_l as isize), r.offset(-1));
                    r = r.offset(-1);
                }
            }
            width(v.as_mut_ptr(), r)
        } else if start_r < end_r {
            debug_assert_eq!(width(l, r), block_r);
            while start_r < end_r {
                unsafe {
                    end_r = end_r.offset(-1);
                    ptr::swap(l, r.offset(-(*end_r as isize) - 1));
                    l = l.offset(1);
                }
            }
            width(v.as_mut_ptr(), l)
        } else {
            width(v.as_mut_ptr(), l)
        }
    }
    
    fn partition<T, F>(v: &mut [T], pivot: usize, is_less: &mut F) -> (usize, bool)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let (mid, was_partitioned) = {
            v.swap(0, pivot);
            let (pivot, v) = v.split_at_mut(1);
            let pivot = &mut pivot[0];
    
            let tmp = mem::ManuallyDrop::new(unsafe { ptr::read(pivot) });
            let _pivot_guard = CopyOnDrop { src: &*tmp, dest: pivot };
            let pivot = &*tmp;
    
            let mut l = 0;
            let mut r = v.len();
    
            unsafe {
                while l < r && is_less(v.get_unchecked(l), pivot) {
                    l += 1;
                }
    
                while l < r && !is_less(v.get_unchecked(r - 1), pivot) {
                    r -= 1;
                }
            }
    
            (l + partition_in_blocks(&mut v[l..r], pivot, is_less), l >= r)
        };
    
        v.swap(0, mid);
    
        (mid, was_partitioned)
    }
    
    fn partition_equal<T, F>(v: &mut [T], pivot: usize, is_less: &mut F) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        v.swap(0, pivot);
        let (pivot, v) = v.split_at_mut(1);
        let pivot = &mut pivot[0];
    
        let tmp = mem::ManuallyDrop::new(unsafe { ptr::read(pivot) });
        let _pivot_guard = CopyOnDrop { src: &*tmp, dest: pivot };
        let pivot = &*tmp;
    
        let mut l = 0;
        let mut r = v.len();
        loop {
            unsafe {
                while l < r && !is_less(pivot, v.get_unchecked(l)) {
                    l += 1;
                }
    
                while l < r && is_less(pivot, v.get_unchecked(r - 1)) {
                    r -= 1;
                }
    
                if l >= r {
                    break;
                }
    
                r -= 1;
                let ptr = v.as_mut_ptr();
                ptr::swap(ptr.add(l), ptr.add(r));
                l += 1;
            }
        }
    
        l + 1
    }
    
    fn choose_pivot<T, F>(v: &mut [T], is_less: &mut F) -> (usize, bool)
    where
        F: FnMut(&T, &T) -> bool,
    {
        const SHORTEST_MEDIAN_OF_MEDIANS: usize = 50;
        const MAX_SWAPS: usize = 4 * 3;
    
        let len = v.len();
    
        let mut a = len / 4 * 1;
        let mut b = len / 4 * 2;
        let mut c = len / 4 * 3;
    
        let mut swaps = 0;
    
        if len >= 8 {
            let mut sort2 = |a: &mut usize, b: &mut usize| unsafe {
                if is_less(v.get_unchecked(*b), v.get_unchecked(*a)) {
                    ptr::swap(a, b);
                    swaps += 1;
                }
            };
    
            let mut sort3 = |a: &mut usize, b: &mut usize, c: &mut usize| {
                sort2(a, b);
                sort2(b, c);
                sort2(a, b);
            };
    
            if len >= SHORTEST_MEDIAN_OF_MEDIANS {
                let mut sort_adjacent = |a: &mut usize| {
                    let tmp = *a;
                    sort3(&mut (tmp - 1), a, &mut (tmp + 1));
                };
    
                sort_adjacent(&mut a);
                sort_adjacent(&mut b);
                sort_adjacent(&mut c);
            }
    
            sort3(&mut a, &mut b, &mut c);
        }
    
        if swaps < MAX_SWAPS {
            (b, swaps == 0)
        } else {
            v.reverse();
            (len - 1 - b, true)
        }
    }
    
    
    fn partition_at_index_loop<'a, T, F>(
        mut v: &'a mut [T],
        mut index: usize,
        is_less: &mut F,
        mut pred: Option<&'a T>,
    ) where
        F: FnMut(&T, &T) -> bool,
    {
        loop {
            const MAX_INSERTION: usize = 10;
            if v.len() <= MAX_INSERTION {
                insertion_sort(v, is_less);
                return;
            }
    
            let (pivot, _) = choose_pivot(v, is_less);
    
            if let Some(p) = pred {
                if !is_less(p, &v[pivot]) {
                    let mid = partition_equal(v, pivot, is_less);
    
                    if mid > index {
                        return;
                    }
    
                    v = &mut v[mid..];
                    index = index - mid;
                    pred = None;
                    continue;
                }
            }
    
            let (mid, _) = partition(v, pivot, is_less);
    
            let (left, right) = v.split_at_mut(mid);
            let (pivot, right) = right.split_at_mut(1);
            let pivot = &pivot[0];
    
            if mid < index {
                v = right;
                index = index - mid - 1;
                pred = Some(pivot);
            } else if mid > index {
                v = left;
            } else {
                return;
            }
        }
    }
    
    fn partition_at_index<T, F>(
        v: &mut [T],
        index: usize,
        mut is_less: F,
    ) -> (&mut [T], &mut T, &mut [T])
    where
        F: FnMut(&T, &T) -> bool,
    {
        use cmp::Ordering::Greater;
        use cmp::Ordering::Less;
    
        if index >= v.len() {
            panic!("partition_at_index index {} greater than length of slice {}", index, v.len());
        }
    
        if mem::size_of::<T>() == 0 {
        } else if index == v.len() - 1 {
            let (max_index, _) = v
                .iter()
                .enumerate()
                .max_by(|&(_, x), &(_, y)| if is_less(x, y) { Less } else { Greater })
                .unwrap();
            v.swap(max_index, index);
        } else if index == 0 {
            let (min_index, _) = v
                .iter()
                .enumerate()
                .min_by(|&(_, x), &(_, y)| if is_less(x, y) { Less } else { Greater })
                .unwrap();
            v.swap(min_index, index);
        } else {
            partition_at_index_loop(v, index, &mut is_less, None);
        }
    
        let (left, right) = v.split_at_mut(index);
        let (pivot, right) = right.split_at_mut(1);
        let pivot = &mut pivot[0];
        (left, pivot, right)
    }

    pub fn select_nth_unstable_by_key<T, K, F>(
        slice:&mut [T],
        index: usize,
        mut f: F,
    ) -> (&mut [T], &mut T, &mut [T])
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        let mut g = |a: &T, b: &T| f(a).lt(&f(b));
        partition_at_index(slice, index, &mut g)
    }
}



mod rnd {
    pub fn next()->usize{
        static mut SEED:usize=88172645463325252;
        unsafe{
            SEED^=SEED<<7;
            SEED^=SEED>>9;
            SEED
        }
    }
}



use std::collections::{HashMap,HashSet};
use core::hash::BuildHasherDefault;
use core::hash::Hasher;

#[derive(Default)]
pub struct NopHasher{
    hash:u64,
}
impl Hasher for NopHasher{
    fn write(&mut self,_:&[u8]){
        panic!();
    }

    #[inline]
    fn write_u64(&mut self,n:u64){
        self.hash=n;
    }

    #[inline]
    fn finish(&self)->u64{
        self.hash
    }
}

pub type NopHashMap<K,V>=HashMap<K,V,BuildHasherDefault<NopHasher>>;
pub type NopHashSet<V>=HashSet<V,BuildHasherDefault<NopHasher>>;