use super::development as image_dev;

use plygui_win32::common::*;

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new("STATIC").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Image = Member<Control<ImageWin32>>;

#[repr(C)]
pub struct ImageWin32 {
    base: WindowsControlBase<Image>,

    bmp: windef::HBITMAP,
    scale: super::ScalePolicy,
}

impl ImageWin32 {
    fn install_image(&mut self, content: super::image::DynamicImage) {
        use image::GenericImage;

        let (w, h) = content.dimensions();

        let bminfo = wingdi::BITMAPINFO {
            bmiHeader: wingdi::BITMAPINFOHEADER {
                biSize: mem::size_of::<wingdi::BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                biHeight: h as i32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: wingdi::BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: unsafe { mem::zeroed() },
        };

        unsafe {
            let mut pv_image_bits = ptr::null_mut();
            let hdc_screen = winuser::GetDC(ptr::null_mut());
            self.bmp = wingdi::CreateDIBSection(hdc_screen, &bminfo, wingdi::DIB_RGB_COLORS, &mut pv_image_bits, ptr::null_mut(), 0);
            winuser::ReleaseDC(ptr::null_mut(), hdc_screen);
            if self.bmp.is_null() {
                panic!("Could not load image.")
            }

            ptr::copy(content.flipv().to_rgba().into_raw().as_ptr(), pv_image_bits as *mut u8, (w * h * 4) as usize);
        }
    }
    fn remove_image(&mut self) {
        unsafe {
            wingdi::DeleteObject(self.bmp as *mut c_void);
        }
        self.bmp = ptr::null_mut();
    }
}

impl Drop for ImageWin32 {
    fn drop(&mut self) {
        self.remove_image();
    }
}

impl image_dev::ImageInner for ImageWin32 {
    fn with_content(content: super::image::DynamicImage) -> Box<super::Image> {
        let mut i = Box::new(Member::with_inner(
            Control::with_inner(
                ImageWin32 {
                    base: WindowsControlBase::new(),

                    bmp: ptr::null_mut(),
                    scale: super::ScalePolicy::FitCenter,
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));

        i.as_inner_mut().as_inner_mut().install_image(content);
        i
    }
    fn set_scale(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, policy: super::ScalePolicy) {
        if self.scale != policy {
            self.scale = policy;
            self.base.invalidate();
        }
    }
    fn scale(&self) -> super::ScalePolicy {
        self.scale
    }
}

impl ControlInner for ImageWin32 {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(member, control, pw, ph);
            create_control_hwnd(
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                self.base.hwnd,
                0,
                WINDOW_CLASS.as_ptr(),
                "",
                winuser::SS_BITMAP | winuser::SS_CENTERIMAGE | winuser::WS_TABSTOP,
                selfptr,
                Some(handler),
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &controls::Container) {
        destroy_hwnd(self.base.hwnd, self.base.subclass_id, Some(handler));
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    fn parent(&self) -> Option<&controls::Member> {
        self.base.parent().map(|p| p.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.parent_mut().map(|p| p.as_member_mut())
    }
    fn root(&self) -> Option<&controls::Member> {
        self.base.root().map(|p| p.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.root_mut().map(|p| p.as_member_mut())
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut development::MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        fill_from_markup_base!(self, base, markup, registry, Image, ["Image"]);
        //TODO image source
    }
}

impl HasLayoutInner for ImageWin32 {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl MemberInner for ImageWin32 {
    type Id = Hwnd;

    fn size(&self) -> (u16, u16) {
        self.base.size()
    }
    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            unsafe {
                winuser::ShowWindow(self.base.hwnd, if base.visibility == types::Visibility::Visible { winuser::SW_SHOW } else { winuser::SW_HIDE });
            }
            self.base.invalidate();
        }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl Drawable for ImageWin32 {
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        self.base.draw(coords);
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, w: u16, h: u16) -> (u16, u16, bool) {
        let old_size = self.base.measured_size;
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => w,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_width
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => h,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_height
                    }
                };
                (cmp::max(0, w as i32) as u16, cmp::max(0, h as i32) as u16)
            }
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

/*
#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
	use super::NewImage;
	
    Image::with_content().into_control()
}
*/

unsafe extern "system" fn handler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let sc: &mut Image = mem::transmute(param);
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as isize);
    }
    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            sc.call_on_resize(width, height);
        }
        winuser::WM_PAINT => {
            let sc = sc.as_inner_mut().as_inner_mut();
            let (pw, ph) = sc.size();
            let mut hoffs = DEFAULT_PADDING;
            let mut voffs = DEFAULT_PADDING;
            let mut hdiff = hoffs + DEFAULT_PADDING;
            let mut vdiff = voffs + DEFAULT_PADDING;
            let inner_h = pw as i32 - hdiff;
            let inner_v = ph as i32 - vdiff;

            let mut bm: wingdi::BITMAP = mem::zeroed();
            let mut ps: winuser::PAINTSTRUCT = mem::zeroed();

            let mut hdc = winuser::BeginPaint(hwnd, &mut ps);
            let mut hdc_mem = wingdi::CreateCompatibleDC(hdc);
            wingdi::SelectObject(hdc_mem, sc.bmp as *mut c_void); //let hbm_old =
            wingdi::GetObjectW(sc.bmp as *mut c_void, mem::size_of::<wingdi::BITMAP>() as i32, &mut bm as *mut _ as *mut c_void);

            let (wrate, hrate) = (inner_h as f32 / bm.bmWidth as f32, inner_v as f32 / bm.bmHeight as f32);
            let less_rate = fmin(wrate, hrate);

            let blendfunc = wingdi::BLENDFUNCTION {
                BlendOp: 0,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: 1,
            };

            let (dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h) = match sc.scale {
                super::ScalePolicy::FitCenter => {
                    let bm_h = (bm.bmWidth as f32 * less_rate) as i32;
                    let bm_v = (bm.bmHeight as f32 * less_rate) as i32;
                    let xoffs = (pw as i32 - bm_h) / 2;
                    let yoffs = (ph as i32 - bm_v) / 2;
                    (xoffs, yoffs, bm_h, bm_v, 0, 0, bm.bmWidth, bm.bmHeight)
                }
                super::ScalePolicy::CropCenter => {
                    let half_diff_h = (bm.bmWidth - pw as i32) / 2;
                    let half_diff_v = (bm.bmHeight - ph as i32) / 2;
                    (
                        hoffs + cmp::min(hoffs, half_diff_h).abs(),
                        voffs + cmp::min(voffs, half_diff_v).abs(),
                        cmp::min(pw as i32, bm.bmWidth),
                        cmp::min(ph as i32, bm.bmHeight),
                        cmp::max(0, half_diff_h),
                        cmp::max(0, half_diff_v),
                        cmp::min(bm.bmWidth, inner_h),
                        cmp::min(bm.bmHeight, inner_v),
                    )
                }
            };
            println!("{}/{}/{}/{} s {}/{}/{}/{}", dst_x, dst_y, dst_w, dst_h, src_x, src_y, src_w, src_h);
            wingdi::GdiAlphaBlend(hdc, dst_x, dst_y, dst_w, dst_h, hdc_mem, src_x, src_y, src_w, src_h, blendfunc);

            wingdi::DeleteDC(hdc_mem);
            winuser::EndPaint(hwnd, &ps);
        }
        _ => {}
    }

    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}

fn fmin(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}
/*fn fmax(a: f32, b: f32) -> f32 {
    // leave for future non-centered fit
    if a > b {
        a
    } else {
        b
    }
}*/

impl_all_defaults!(Image);
