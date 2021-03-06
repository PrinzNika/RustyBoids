extern crate rand;  
use nannou::prelude::*;
use rand::Rng;
use nannou_osc as osc;

struct Model {
    boids: Vec<Boid>,
    sender: osc::Sender<osc::Connected>,
}

#[derive(Clone, Debug)]
struct Boid {
    position: Vector2,
    velocity: Vector2,
}

impl Model {
    fn velocity_update(&mut self, deltat:f32) {
        let center = perceived_center(&self.boids);
        let velocity = perceived_velocity(&self.boids);
        let length = self.boids.len() as f32;
        
        let boids = self.boids.clone();
        let collision = collision_avoidance(&boids);

        for boid in &mut self.boids {
            let c =  center/length - boid.position;
            boid.velocity += c  * deltat * 0.05; 
        }
        for boid in & mut self.boids {
            let mut umgebende = Vec::new();
            

            for individual in &boids {
                    if (individual.position - boid.position).magnitude() < 150.0 { 
                        umgebende.push(individual.clone());  
                }
            }

            let velocity = perceived_velocity(&umgebende);
            boid.velocity += (( velocity - boid.velocity) / (length - 1.0)) / 10.0 * deltat;
        }
        for index in 0..self.boids.len() {
            self.boids[index].velocity += collision[index] * deltat * 40.0;
        }
        for boid in &mut self.boids {

            boid.velocity -= boid.position / 5000.0;
        }
        
        for boid in &mut self.boids {
            if boid.velocity.magnitude() > 100. {
                boid.velocity *= 0.99;
            }
            if boid.velocity.magnitude() < 30. {
                boid.velocity *= 1.1;
            }
        }
        
    } 
    fn position_update(&mut self, deltat:f32) {
        for boid in & mut self.boids {
            boid.position += boid.velocity * deltat;
        }
    }
    fn control_update(&mut self) {
        let mut number = 1 ;
        for boid in &self.boids {
            send_boid(&boid.position, &self.sender, number);
            number += 1;
        }
    
    }
}

fn send_boid(pos: &Vector2, control: &osc::Sender<osc::Connected> , number: i32) {
    let osc_addr1 = format!("/boid/angle{}", number);
    let osc_addr2 = format!("/boid/length{}", number);
    let angle = pos.y.atan2(pos.x);
    let length = pos.magnitude() as f32;
    

    let arg1 = vec![osc::Type::Float(angle)];
    let arg2 = vec![osc::Type::Float(length)];
    let packet1 = (osc_addr1, arg1);
    let packet2 = (osc_addr2, arg2);
    control.send(packet1).ok();
    control.send(packet2).ok();
}

fn v2(a: f32, b: f32) -> Vector2 {
    Vector2::new(a, b)
}

fn draw_boid(pos: &Vector2, velocity: &Vector2, draw: &Draw) {
    draw.arrow()
        .color(STEELBLUE)
        .points(*pos, *pos + *velocity)
        .weight(5.0);
}


fn spawn_boids() -> Vec<Boid> {
    let number = 15;
    let mut position = v2(0.0,0.0); 
    let mut velocity = v2(0.0,0.0); 
    let mut rng = rand::thread_rng();
    let mut vec = Vec::new();
    

    for i in 0..number {
        position = v2(rng.gen_range(-1000.0,1000.0),rng.gen_range(-1000.0,1000.0));
        velocity = v2(rng.gen_range(-1000.0,1000.0),rng.gen_range(-1000.0,1000.0));
        // insert into model vector
        vec.push(
            Boid {
            position: position,
            velocity: velocity,
            }
        );
    }    
    vec
}


fn perceived_center(schwarm: &Vec<Boid>) -> Vector2 {
    let mut x = 0.0;
    let mut y = 0.0;
    for boid in schwarm {
        x += boid.position.x;
        y += boid.position.y;
    }
    v2(x, y)
}

fn perceived_velocity(schwarm: &Vec<Boid>) -> Vector2 {

    let mut x = 0.0;
    let mut y = 0.0;
    for boid in schwarm {
        x += boid.velocity.x;
        y += boid.velocity.y;
    }
    v2(x, y)
}

fn collision_avoidance(schwarm: &Vec<Boid>) -> Vec<Vector2> {
    let mut directions = vec![];

    for boid in schwarm {
        let mut direction = Vector2::new(0.0,0.0);
        for individual in schwarm {
            if ! std::ptr::eq(boid, individual) {
                if (individual.position - boid.position).magnitude() < 50.0 { 
                    direction -= (individual.position - boid.position)*(1./(individual.position - boid.position).magnitude()); // stärker je näher
                }
            }
        }
        directions.push(direction);
    }
    directions
}


    

    

fn control() -> osc::Sender<osc::Connected> {
    let port = 5510;
    let target_addr = format!("{}:{}", "127.0.0.1", port);

    let sender = osc::sender()
        .expect("Could not bind to default socket")
        .connect(target_addr)
        .expect("Could not connect to socket at address");

    sender
}






fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let instanz = spawn_boids();
    let control = control();
    Model { boids: instanz,
            sender: control,
    }
}



fn event(app: &App, model: &mut Model, event: Event) {
    if let Event::Update(u) = event {
        let deltat = u.since_last.as_secs_f32();
        model.velocity_update(deltat);
        model.position_update(deltat);

        model.control_update();
    }
    
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Generate sine wave data based on the time of the app

    // Clear the background to purple.
    draw.background().color(PLUM);

    for boid in &model.boids {
        draw_boid(&boid.position, &boid.velocity, &draw);
    }


    draw.to_frame(app, &frame).unwrap();
}
