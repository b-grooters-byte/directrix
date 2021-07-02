use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};
use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;

const PARABOLA_X_STEP: usize = 5;
const WINDOW_INIT_WIDTH: i32 = 600;
const WINDOW_INIT_HEIGHT: i32 = 400;

/// Site is a tuple struct that represents the current state of a simplified
/// Voronoi site. The fields are :
/// * 0 - X Position
/// * 1 - Y Position
/// * 2 - Directrix Y
#[derive(Debug, Copy, Clone)]
struct Site(f64, f64, f64);

impl Site {
    /// Renders the current directrix based representation of a site.
    pub fn draw(&self, width: i32, height: i32, ctx: &cairo::Context) {
        // get the clip region
        let clip = ctx.clip_extents().unwrap();
        // draw the origin
        ctx.set_source_rgba(1.0, 0.0, 0.0, 1.0);
        ctx.new_path();
        ctx.arc(self.0, self.1, 2.0, 0.0, 2.0 * std::f64::consts::PI);
        if let Err(_e) = ctx.stroke() {
            // TODO handle error
        }
        // draw the directrix
        ctx.set_source_rgba(0.0, 0.0, 0.0, 1.0);
        ctx.new_path();
        ctx.move_to(1.0, self.2);
        ctx.line_to(width as f64, self.2);
        if let Err(_e) = ctx.stroke() {
            // TODO handle error
        }
        // draw the parabola
        ctx.set_source_rgba(0.0, 0.0, 0.0, 1.0);
        ctx.new_path();
        // used to start and stop the line_to once the arc is outside the visible window
        let mut rendering = false;
        let mut stop_render = false;
        let mut prev_x: usize = 0;
        let mut prev_y: Option<f64> = None;
        ctx.set_source_rgba(0.0, 1.0, 0.0, 1.0);
        ctx.new_path();
        let start_x = cmp::max(clip.0 as i32 - PARABOLA_X_STEP as i32, 0) as usize;
        let end_x = clip.2 as usize + PARABOLA_X_STEP;
        for x in (start_x..=end_x).step_by(PARABOLA_X_STEP) {
            let y = 1.0 / (2.0 * (self.1 - self.2)) * ((x as f64 - self.0) * (x as f64 - self.0))
                + ((self.1 + self.2) / 2.0);
            if rendering {
                ctx.line_to(x as f64, y);
            }
            if y > 0.0 && y < clip.3 as f64 && !rendering {
                rendering = true;
                if let Some(y) = prev_y {
                    ctx.move_to(prev_x as f64, y);
                } else {
                    ctx.move_to(x as f64, y);
                }
            } else {
                prev_x = x;
                prev_y = Some(y);
            }
            if stop_render {
                break;
            }
            stop_render = rendering && (y < 0.0 || y > height as f64);
        }
        if let Err(_e) = ctx.stroke() {
            // TODO handle error
        }
        // render the text
        ctx.set_source_rgba(0.0, 0.0, 0.0, 1.0);
        ctx.select_font_face(
            "Monospace",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        ctx.set_font_size(12.0);
        ctx.move_to(8.0, 20.0);
        if let Err(_e) = ctx.show_text(format!("   Focus: [{}, {}]", self.0, self.1).as_str()) {
            // TODO handle eror
        }
        ctx.move_to(8.0, 34.0);
        if let Err(_e) = ctx.show_text(format!("Directrix: {}", self.2).as_str()) {
            // TODO handle eror
        }
    }
}

/// Application entry point
fn main() {
    // default site location
    let site = Site {
        0: WINDOW_INIT_WIDTH as f64 / 2.0,
        1: WINDOW_INIT_HEIGHT as f64 / 2.0,
        2: WINDOW_INIT_HEIGHT as f64 / 2.0 + 10.0,
    };
    // create a new application
    let app = Application::new(Some("org.bytetrail.dtx"), Default::default());
    app.connect_activate(move |app| {
        build_ui(app, site);
    });
    // run the application
    app.run();
}

/// Builds the GTK UI with drawing area.
fn build_ui(app: &Application, site: Site) {
    // create the window
    let window = ApplicationWindow::new(app);
    window.set_title("Directrix");
    let canvas = DrawingArea::builder()
        .events(
            gdk::EventMask::POINTER_MOTION_MASK
                | gdk::EventMask::BUTTON_PRESS_MASK
                | gdk::EventMask::BUTTON_RELEASE_MASK
                | gdk::EventMask::EXPOSURE_MASK,
        )
        .expand(true)
        .width_request(WINDOW_INIT_WIDTH)
        .height_request(WINDOW_INIT_HEIGHT)
        .build();
    // create a reference counting smart pointer so that site may be passed to
    // to each event closure. These all occur on the UI thread so Arc not
    // necessary
    let site_rc = Rc::new(RefCell::new(site));
    let site_clone = Rc::clone(&site_rc);
    // handle the draw request for DrawingArea
    canvas.connect_draw(move |area, ctx| {
        let s = site_clone.borrow();
        s.draw(area.allocated_width(), area.allocated_height(), ctx);
        Inhibit(false)
    });
    // handle the mouse button release for DrawingArea
    let site_clone = Rc::clone(&site_rc);
    canvas.connect_button_release_event(move |area, event| {
        let (x, y) = event.coords().unwrap();
        let mut s = site_clone.borrow_mut();
        s.0 = x;
        s.1 = y;
        area.queue_draw();
        Inhibit(false)
    });
    // handle the mouse motion event for DrawingArea
    let site_clone = Rc::clone(&site_rc);
    canvas.connect_motion_notify_event(move |area, event| {
        let (_x, y) = event.coords().unwrap();
        let mut s = site_clone.borrow_mut();
        s.2 = y;
        area.queue_draw();
        Inhibit(true)
    });
    window.add(&canvas);
    window.show_all();
}
