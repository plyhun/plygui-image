use super::development as image_dev;

use plygui_api::{layout, types, utils, controls};
use plygui_api::development::*;	

use plygui_gtk::common;

use gtk::{Cast, Widget, WidgetExt, Image as GtkImageSys, Bin, BinExt, Label, LabelExt, CssProvider, CssProviderExt, StyleContextExt};
use gdk_pixbuf::{Pixbuf, Colorspace};
use pango::LayoutExt;
use cairo::Format;

use std::cmp::max;

pub type Image = Member<Control<GtkImage>>;

#[repr(C)]
pub struct GtkImage {
    base: common::GtkControlBase<Image>,
    
    scale: super::ScalePolicy,
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
            }, ()), MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        
        {
        	let ptr = i.as_ref() as *const _ as *mut ::std::os::raw::c_void;
        	i.as_inner_mut().as_inner_mut().base.set_pointer(ptr);
        }
        i.as_inner_mut().as_inner_mut().base.widget.connect_size_allocate(on_size_allocate);
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
    fn apply_padding(&mut self, base: &ControlBase) {
	    let (lp,tp,rp,bp) = base.layout.padding.into();
			
		let self_widget: Widget = self.base.widget.clone().into();	
	    let btn = self_widget.downcast::<GtkImageSys>().unwrap();   
		let css = CssProvider::new();
		css.load_from_data(format!("GtkImage {{ padding-left: {}px; padding-top: {}px; padding-right: {}px; padding-bottom: {}px; }}", lp, tp, rp, bp).as_bytes()).unwrap();
		btn.get_style_context().unwrap().add_provider(&css, ::gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

impl HasLayoutInner for GtkImage {
	fn on_layout_changed(&mut self, base: &mut MemberBase) {
		self.apply_padding(unsafe { &mut utils::member_control_base_mut_unchecked(base).control });
		self.base.invalidate();
	}
}

impl ControlInner for GtkImage {
	fn on_added_to_container(&mut self, base: &mut MemberControlBase, parent: &controls::Container, x: i32, y: i32) {
		let (pw, ph) = parent.draw_area_size();
        self.measure(base, pw, ph);
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

fn on_size_allocate(this: &::gtk::Widget, _allo: &::gtk::Rectangle) {
	let mut ll = this.clone().upcast::<Widget>();
	let ll = common::cast_gtk_widget_to_member_mut::<Image>(&mut ll).unwrap();
	
	let measured_size = ll.as_inner().as_inner().base.measured_size;
	if let Some(ref mut cb) = ll.base_mut().handler_resize {
        let mut w2 = this.clone().upcast::<Widget>();
		let mut w2 = common::cast_gtk_widget_to_member_mut::<Image>(&mut w2).unwrap();
		(cb.as_mut())(w2, measured_size.0 as u16, measured_size.1 as u16);
    }
}

impl_all_defaults!(Image);
