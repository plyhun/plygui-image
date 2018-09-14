use super::development as image_dev;

use plygui_cocoa::common::*;

use plygui_cocoa::core_graphics::base::{kCGBitmapByteOrderDefault, kCGImageAlphaLast};
use plygui_cocoa::core_graphics::color_space::CGColorSpace;
use plygui_cocoa::core_graphics::data_provider::CGDataProvider;
use plygui_cocoa::core_graphics::image::CGImage;

use std::sync::Arc;

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        common::register_window_class("PlyguiImage", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const DEFAULT_PADDING: i32 = 6;
const BASE_CLASS: &str = "NSImageView";

pub type Image = Member<Control<ImageCocoa>>;

#[repr(C)]
pub struct ImageCocoa {
    base: common::CocoaControlBase<Image>,

    img: cocoa_id,
}

impl ImageCocoa {
    fn install_image(&mut self, content: super::image::DynamicImage) {
        use image::GenericImage;

        let size = content.dimensions();

        unsafe {
            let color_space = CGColorSpace::create_device_rgb();
            let provider = CGDataProvider::from_buffer(Arc::new(content.to_rgba().into_raw()));
            let cgimage = CGImage::new(size.0 as usize, size.1 as usize, 8, 32, 4 * size.0 as usize, &color_space, kCGBitmapByteOrderDefault | kCGImageAlphaLast, &provider, true, 0);

            self.img = msg_send![class!(NSImage), alloc];
            let size = NSSize::new(size.0 as f64, size.1 as f64);
            let () = msg_send![self.img, initWithCGImage:cgimage size:size];
            let () = msg_send![self.base.control, setImage:self.img];
        }
    }
    fn remove_image(&mut self) {
        unsafe {
            let () = msg_send![self.img, dealloc];
        }
    }
}

impl Drop for ImageCocoa {
    fn drop(&mut self) {
        self.remove_image();
    }
}

impl image_dev::ImageInner for ImageCocoa {
    fn with_content(content: super::image::DynamicImage) -> Box<super::Image> {
        let mut i = Box::new(Member::with_inner(
            Control::with_inner(
                ImageCocoa {
                    base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                    img: nil,
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = i.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *i.as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
            let () = msg_send![i.as_inner_mut().as_inner_mut().base.control, setImageAlignment:0];
        }
        i.as_inner_mut().as_inner_mut().install_image(content);
        i
    }
    fn set_scale(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, policy: super::ScalePolicy) {
        if self.scale() != policy {
            let scale = policy_to_nsscale(policy);
            unsafe {
                let () = msg_send![self.base.control, setImageScaling: scale];
            }
            self.base.invalidate();
        }
    }
    fn scale(&self) -> super::ScalePolicy {
        let scale = unsafe { msg_send![self.base.control, imageScaling] };
        nsscale_to_policy(scale)
    }
}

impl ControlInner for ImageCocoa {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        self.base.invalidate();
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &controls::Container) {
        unsafe {
            self.base.on_removed_from_container();
        }
    }

    fn parent(&self) -> Option<&controls::Member> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&controls::Member> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.root_mut()
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        fill_from_markup_base!(self, base, markup, registry, Image, ["Image"]);
        //TODO image source
    }
}

impl MemberInner for ImageCocoa {
    type Id = common::CocoaId;

    fn size(&self) -> (u16, u16) {
        self.base.size()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        self.base.on_set_visibility(base);
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasLayoutInner for ImageCocoa {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for ImageCocoa {
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        self.base.draw(coords);
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = self.base.measured_size;
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        label_size = common::measure_nsstring(msg_send![self.base.control, title]);
                        label_size.0 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            label_size = common::measure_nsstring(msg_send![self.base.control, title]);
                        }
                        label_size.1 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

fn policy_to_nsscale(i: super::ScalePolicy) -> u32 {
    // TODO NSImageScaling
    match i {
        super::ScalePolicy::CropCenter => 2,
        super::ScalePolicy::FitCenter => 3,
    }
}
fn nsscale_to_policy(i: u32) -> super::ScalePolicy {
    match i {
        3 => super::ScalePolicy::FitCenter,
        2 => super::ScalePolicy::CropCenter,
        _ => {
            println!("Unknown scale: {}", i);
            super::ScalePolicy::FitCenter
        }
    }
}
/*#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Image::with_label("").into_control()
}*/

extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Image>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_resize(param.width as u16, param.height as u16)
    }
}
impl_all_defaults!(Image);
