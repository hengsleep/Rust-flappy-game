use std::fmt::format;//标准库中的std::fmt模块中的format函数

use bracket_lib::prelude::*;//导入了bracket-lib库中的prelude模块中所有内容
//可以在代码中直接使用 BTermBuilder、GameState 等等而不用写完整的路径。

enum GameMode {//定义枚举（enumeration）类型
    Menu,
    Playing,
    End,
}

//const不变值
const SCREEN_WIDTH: i32 = 80;//i32表示有符号的32位整数，相当于c中的int
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;//75毫秒作为单位时间

struct Player {//struct结构体
    x: i32,//水平位置,整数类型，世界空间里的横坐标可以是无穷大
    y: i32,//垂直位置,整数类型
    velocity: f32,//垂直方向纵向速度，浮点数类型
}

impl Player {//实现（implement）将方法与结构体、枚举或trait关联起来，类似于class关键字用于定义类
    fn new(x: i32, y: i32) -> Self {//Self表示当前方法所属的结构体类型，返回更调用new函数的类型相同的类型
        Player {
            x: 0,//初始化
            y: 0,
            velocity: 0.0,
        }    
    }

    fn render(&mut self, ctx: &mut BTerm) {//render渲染，&表示引用
        //self为该render调用者，ctx context上下文引用BTerm类型的上下文
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));//将字符'@'转换为CP437编码，用于表示字符的不同图形表示。
    }//通过self.y，我们可以访问当前Player实例self的y字段，即当前玩家对象的垂直位置。
    
    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }//重力加速度，每次加0.2，直至2.0

        self.y += self.velocity as i32;
        self.x += 1;//player在x轴水平方向上不动

        if self.y < 0 {//向上为负
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;//往上飞相当于速度为负
    }
}

struct State {
    mode: GameMode,//GameMode结构体，Menu,Playing,End
    player: Player,
    frame_time: f32,//表示游戏经过多少帧以后，累计了多少时间
    obstacle: Obstacle,
    score: i32,
}

//状态的关联函数
impl State {
    fn new() -> Self {
        State {
            player: Player::new(10, 25),
            frame_time: 0.0,
            mode: GameMode::Menu,//游戏的初始状态为菜单
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);//清理屏幕，指定背景颜色

        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);//渲染部分
        ctx.print(0, 0, "Press space to Flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x);

        if self.player.x > self.obstacle.x {//player横坐标大于障碍物横坐标，加一分
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);//下一个障碍物更难
        }

        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {//纵坐标超出了屏幕的高度，坠地游戏结束
            self.mode = GameMode::End;//模式转为结束模式
        }
    }
    
    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;//模式转为游戏中模式
        self.score = 0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();//清理屏幕
        ctx.print_centered(5, "Welcome to Flappy Game! -Xheng1934");//5是纵坐标，水平居中
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }            
        }
    }
    
    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();//清理屏幕
        ctx.print_centered(5, "You are dead!");//5是纵坐标，水平居中
        ctx.print_centered(6, &format!("You earned {} points", self.score));//得分
        ctx.print_centered(8, "(P) Play Again ");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }            
        }
    }
}

//坐标系从左上角开始为(0,0)，往下和往右都是正
impl GameState for State {
    fn tick(&mut self,ctx: &mut BTerm) {
        //判断游戏当前的状态
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
        //ctx.cls();
        //ctx.print(1, 1, "Hello, Bracket termial!");
    }
}

struct Obstacle {
    x: i32,//世界空间里的横坐标可以是无穷大
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),//障碍的中心点随机
            size: i32::max(2,20 - score), // 这里假设障碍物的尺寸为固定值 10，您可以根据需求修改
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x: i32 = self.x - player_x;//屏幕空间，障碍在世界空间里的横坐标减去玩家在世界空间里的横坐标
        let half_size: i32 = self.size / 2;
    
        for y in 0..self.gap_y - half_size {//for循环a..b从a到b
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));//screen_x渲染屏幕的横坐标，而非世界横坐标
        }
    
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }
    
    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size: i32 = self.size / 2;
        let does_x_match: bool = player.x == self.x;
        let player_above_gap: bool = player.y < self.gap_y - half_size;
        let player_below_gap: bool = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

fn main() -> BError {//表示main()函数可能返回一个BError类型的错误
    let context = BTermBuilder::simple80x50()//let用于声明一个不可变变量，let mut表示可变变量
        .with_title("Flappy Game")//exe程序标题
        .build()?;//?问号表示可能出错，出错返回BError
    
    main_loop(context, State::new())
    //bracket-lib游戏库提供了main_loop函数，控制游戏的主循环，参数是context和state
    //rust的函数里最后一个表达式的值会成为函数的返回值，返回游戏状态，因此无分号

}
