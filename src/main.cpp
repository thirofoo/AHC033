#include <bits/stdc++.h>
using namespace std;
#if __has_include(<atcoder/all>)
    #include <atcoder/all>
using namespace atcoder;
#endif
typedef long long ll;
typedef pair<int, int> P;
typedef tuple<int, int, int, int> T;
#define rep(i, n) for(int i = 0; i < n; i++)

namespace utility {
    struct timer {
        chrono::system_clock::time_point start;
        // 開始時間を記録
        void CodeStart() {
            start = chrono::system_clock::now();
        }
        // 経過時間 (ms) を返す
        double elapsed() const {
        using namespace std::chrono;
            return (double)duration_cast<milliseconds>(system_clock::now() - start).count();
        }
    } mytm;
}

inline unsigned int rand_int() {
    static unsigned int tx = 123456789, ty=362436069, tz=521288629, tw=88675123;
    unsigned int tt = (tx^(tx<<11));
    tx = ty; ty = tz; tz = tw;
    return ( tw=(tw^(tw>>19))^(tt^(tt>>8)) );
}

inline double rand_double() {
    return (double)(rand_int()%(int)1e9)/1e9;
}

inline double gaussian(double mean, double stddev) {
    // 標準正規分布からの乱数生成（Box-Muller変換
    double z0 = sqrt(-2.0 * log(rand_double())) * cos(2.0 * M_PI * rand_double());
    // 平均と標準偏差の変換
    return mean + z0 * stddev;
}

//温度関数
#define TIME_LIMIT 2950
inline double temp(double start) {
    double start_temp = 100,end_temp = 1;
    return start_temp + (end_temp-start_temp)*((utility::mytm.elapsed()-start)/TIME_LIMIT);
}

//焼きなましの採用確率
inline double prob(int best,int now,int start) {
    return exp((double)(now - best) / temp(start));
}

//-----------------以下から実装部分-----------------//

// right | down | left | up
#define DIR_NUM 4
vector<int> dx = { 0, 1, 0,-1};
vector<int> dy = { 1, 0,-1, 0};

inline bool outField(int x,int y,int h,int w) {
    if(0 <= x && x < h && 0 <= y && y < w)return false;
    return true;
}

struct State{

    State() {

    }
};

struct Solver{

    Solver() {
        this->input();

    }

    void input() {
        

        return;
    }

    void output() {

        return;
    }

    void solve() {

        return;
    }
};

int main() {
    cin.tie(0);
    ios_base::sync_with_stdio(false);

    Solver solver;
    solver.solve();
    solver.output();
    
    return 0;
}