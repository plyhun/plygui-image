use super::development as image_dev;

use plygui_api::{layout, types, utils, controls};
use plygui_api::development::*;	

use plygui_gtk::common;

use gtk::{Cast, Widget, WidgetExt, Image as GtkImageSys, ImageExt, Bin, BinExt, Label, LabelExt};
use gdk_pixbuf::{Pixbuf, PixbufExt, Colorspace, InterpType};
use pango::LayoutExt;
use cairo::Format;

use std::cmp::{min, max};

pub type Image = Member<Control<GtkImage>>;

#[repr(C)]
pub struct GtkImage {
    base: common::GtkControlBase<Image>,
    
    scale: super::ScalePolicy,
    orig: Pixbuf,
}

impl image_dev::ImageInner for GtkImage {
    fn with_content(content: super::image::DynamicImage) -> Box<super::Image> {
        use image::GenericImage;
		
		let (w, h) = content.dimensions();
		let raw = content.to_rgba().into_raw();
		let stride = Format::ARgb32.stride_for_width(w).unwrap();
		
		let pixbuf = Pixbuf::new_from_vec(raw, Colorspace::Rgb, true, 8, w as i32, h as i32, stride);        
        
        let mut i = Box::new(Member::with_inner(Control::with_inner(GtkImage {
                base: common::GtkControlBase::with_gtk_widget(GtkImageSys::new_from_pixbuf(Some(&pixbuf)).upcast::<Widget>()),
                scale: super::ScalePolicy::FitCenter,  
                orig: pixbuf,
            }, ()), MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        
        i.as_inner_mut().as_inner_mut().base.widget.connect_size_allocate(on_size_allocate);
        i.as_inner_mut().as_inner_mut().base.widget.connect_show(on_show);
        {
        	let ptr = i.as_ref() as *const _ as *mut ::std::os::raw::c_void;
        	i.as_inner_mut().as_inner_mut().base.set_pointer(ptr);
        }
        i
    }
	fn set_scale(&mut self, _: &mut MemberControlBase, policy: super::ScalePolicy) {
		if self.scale != policy {
			self.scale = policy;
			self.base.invalidate();
		}
	}
    fn scale(&self) -> super::ScalePolicy {
    	self.scale
    }
}

impl GtkImage {
    fn apply_sized_image(&mut self, base: &MemberControlBase) {
        let bm_width = self.orig.get_width();
    	let bm_height = self.orig.get_height();
    	
    	let (lp, tp, rp, bp) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        let (aw, ah) = self.base.measured_size;
        let hoffs = lp + lm;
        let voffs = tp + tm;
        let hdiff = hoffs + rp + rm;
        let vdiff = voffs + bp + bm;
        let inner_h = aw as i32 - hdiff;
    	let inner_v = ah as i32 - vdiff;
    
    	let (wrate, hrate) = (inner_h as f32 / bm_width as f32, inner_v as f32 / bm_height as f32);
        let less_rate = fmin(wrate, hrate);
        
        println!("{}, {}, {}, {}, {}", aw, ah, hoffs, voffs, less_rate);
        
        let scaled = match self.scale {
    		super::ScalePolicy::FitCenter => {
        		let bm_h = (bm_width as f32 * less_rate) as i32;
            	let bm_v = (bm_height as f32 * less_rate) as i32;
        		let alpha = self.orig.get_has_alpha();
            	let bits = self.orig.get_bits_per_sample();
    	
        		let scaled = Pixbuf::new(Colorspace::Rgb, alpha, bits, bm_h, bm_v);
            	self.orig.scale(&scaled, 0, 0, bm_h, bm_v, 0f64, 0f64, less_rate as f64, less_rate as f64, InterpType::Hyper);
                scaled
    		},
    		super::ScalePolicy::CropCenter => {
    			let half_diff_h = (bm_width - aw as i32) / 2;
    			let half_diff_v = (bm_height - ah as i32) / 2;
        		self.orig.new_subpixbuf(max(0, half_diff_h), max(0, half_diff_v), min(bm_width, inner_h), min(bm_height, inner_v)).unwrap()
    		}
    	};
    	let image: Widget = self.base.widget.clone().into();
    	image.downcast::<GtkImageSys>().unwrap().set_from_pixbuf(&scaled);
    }
}

impl HasLayoutInner for GtkImage {
	fn on_layout_changed(&mut self, _: &mut MemberBase) {
		//self.apply_padding(unsafe { &mut utils::member_control_base_mut_unchecked(base).control });
		self.base.invalidate();
	}
}

impl ControlInner for GtkImage {
	fn on_added_to_container(&mut self, base: &mut MemberControlBase, parent: &controls::Container, x: i32, y: i32) {
		let (pw, ph) = parent.draw_area_size();
        self.measure(base, pw, ph);
        println!("{} {} {:?}", ph, ph, self.base.measured_size);
        //self.apply_sized_image(base);
        self.draw(base, Some((x, y)));
	}
    fn on_removed_from_container(&mut self, _: &mut MemberControlBase, _: &controls::Container) {}
    
    fn parent(&self) -> Option<&controls::Member> {
    	self.base.parent().map(|m| m.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.parent_mut().map(|m| m.as_member_mut())
    }
    fn root(&self) -> Option<&controls::Member> {
    	self.base.root().map(|m| m.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.root_mut().map(|m| m.as_member_mut())
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberControlBase, mberarkup: &super::markup::Markup, registry: &mut super::markup::MarkupRegistry) {
    	use plygui_api::markup::MEMBER_TYPE_BUTTON;
		fill_from_markup_base!(
            self,
            base,
            markup,
            registry,
            Image,
            [MEMBER_TYPE_BUTTON]
        );
        fill_from_markup_label!(self, &mut base.member, markup);
        fill_from_markup_callbacks!(self, markup, registry, [on_click => plygui_api::callbacks::Click]);
    }
}

impl MemberInner for GtkImage {
	type Id = common::GtkWidget;
	
    fn size(&self) -> (u16, u16) {
    	self.base.measured_size
    }
    
    fn on_set_visibility(&mut self, _: &mut MemberBase) {
    	self.base.invalidate()
    }
    
    unsafe fn native_id(&self) -> Self::Id {
    	self.base.widget.clone().into()
    }
}

impl Drawable for GtkImage {
	fn draw(&mut self, base: &mut MemberControlBase, coords: Option<(i32, i32)>) {
		self.base.draw(base, coords);
	}
    fn measure(&mut self, base: &mut MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
    	self.base.measured_size = match base.member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let (lp,tp,rp,bp) = base.control.layout.padding.into();
		    	let (lm,tm,rm,bm) = base.control.layout.margin.into();
		    	    	
		    	let mut label_size = (-1i32, -1i32);
                let w = match base.control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        if label_size.0 < 0 {
                        	let self_widget: Widget = self.base.widget.clone().into();
                        	let mut bin = self_widget.downcast::<Bin>().unwrap();
                        	let mut label = bin.get_child().unwrap().downcast::<Label>().unwrap();		
                        	label_size = label.get_layout().unwrap().get_pixel_size();			
                        }
                        // why the bloody hell I need these?
                        label_size.0 += 4;
                        label_size.1 += 4;
                        
                        label_size.0 + lp + rp + lm + rm
                    } 
                };
                let h = match base.control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 0 {
                        	let self_widget: Widget = self.base.widget.clone().into();
                            let mut bin = self_widget.downcast::<Bin>().unwrap();
                        	let mut label = bin.get_child().unwrap().downcast::<Label>().unwrap();		
                        	label_size = label.get_layout().unwrap().get_pixel_size();	
                        }
                        // why the bloody hell I need these?
                        label_size.0 += 4;
                        label_size.1 += 4;
                        
                        label_size.1 + tp + bp + tm + bm
                    } 
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
    	(
            self.base.measured_size.0,
            self.base.measured_size.1,
            self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self, _: &mut MemberControlBase) {
    	self.base.invalidate()
    }
}

/*#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
	Image::with_label("").into_control()
}*/

fn on_show(this: &::gtk::Widget) {
    let mut ll1 = this.clone().upcast::<Widget>();
    let mut ll2 = this.clone().upcast::<Widget>();
	let ll1 = common::cast_gtk_widget_to_member_mut::<Image>(&mut ll1).unwrap();
	let ll2 = common::cast_gtk_widget_to_member_mut::<Image>(&mut ll2).unwrap();
	
	println!("Shown");
	
	ll1.as_inner_mut().as_inner_mut().apply_sized_image(utils::member_control_base(ll2));
}

fn on_size_allocate(this: &::gtk::Widget, _allo: &::gtk::Rectangle) {
    let mut ll1 = this.clone().upcast::<Widget>();
    let mut ll2 = this.clone().upcast::<Widget>();
	let ll1 = common::cast_gtk_widget_to_member_mut::<Image>(&mut ll1).unwrap();
	let ll2 = common::cast_gtk_widget_to_member_mut::<Image>(&mut ll2).unwrap();
	
	ll1.as_inner_mut().as_inner_mut().apply_sized_image(utils::member_control_base(ll2));
	
	let measured_size = ll1.as_inner().as_inner().base.measured_size;
	if let Some(ref mut cb) = ll1.base_mut().handler_resize {
        (cb.as_mut())(ll2, measured_size.0 as u16, measured_size.1 as u16);
    }
}

fn fmin(a: f32, b: f32) -> f32 {
	if a < b { a } else { b }
}

impl_all_defaults!(Image);
