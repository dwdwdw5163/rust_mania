use piston_window::*;
use rand::*;

const HEIGHT:f64=720.0;
const WIDTH:f64=1280.0;




struct Bubble{
    speed:f64,
    x:f64,
    y:f64,
    r:f64
}

impl Bubble{
    pub fn new(num:Option<f64>) -> Bubble{
	let r = (random::<f64>()*(WIDTH/8.0))+5.0;
	let mut b:Bubble=Bubble{
	    speed:(random::<f64>()*150.0)+10.0,
	    y:random::<f64>()*(HEIGHT+r),
	    x:random::<f64>()*WIDTH,
	    r:r
	};
	if let Some(y)=num{
	    b.speed = 0.0;
	    b.y = y
	};
	b
    }
}

struct App {
    bubbles:Vec<Bubble>
}

impl App {
    fn render(&mut self, e: &Event, window: &mut PistonWindow) {
	window.draw_2d(e, |context, gl, _device| {
	    clear([104.0/255.0,221.0/255.0,19.0/255.0,1.0], gl);
	    for b in self.bubbles.iter_mut() {
		ellipse([1.0,97.0/255.0,0.0,1.0],[b.x-b.r, b.y-b.r, b.r*2.0, b.r*2.0], context.transform, gl);
	    }
	});
    }

    fn update(&mut self,args: &UpdateArgs) {
	for b in self.bubbles.iter_mut() {
	    b.y -= b.speed*args.dt;
	    if b.y+b.r <= 0.0{ b.y = HEIGHT+b.r}
	}
    }
}


fn get_bubbles() -> Vec<Bubble>{
    let mut bubbles=Vec::new();
    let n = (random::<u64>()%15)+10;
    for _ in 0..n{
	//bubbles.push(Bubble::new(Some(HEIGHT)));
	//bubbles.push(Bubble::new(Some(0.0)));
	bubbles.push(Bubble::new(None));
    }
    bubbles
}

fn main() {
    let bub = [1.0,97.0/255.0,0.0,1.0];
    let bg = [104.0/255.0,221.0/255.0,19.0/255.0,1.0];
    let mut app = App {
	bubbles:get_bubbles()
    };
    let mut window:PistonWindow = WindowSettings::new("Lava Lamp",[WIDTH,HEIGHT])
        .exit_on_esc(true).build().unwrap();
    let mut events = window.events;
    while let Some(e) = events.next(&mut window){
	if let Some(_) = e.render_args(){
	    app.render(&e,&mut window);
	};
	if let Some(args) = e.update_args(){
	    app.update(&args);
	}
    }
}
