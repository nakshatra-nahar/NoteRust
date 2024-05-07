// Imports
use crate::{RnAppWindow, RnCanvasWrapper};
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, ToggleButton};
use rnote_engine::pens::pensconfig::toolsconfig::ToolStyle;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/toolspage.ui")]
    pub(crate) struct RnToolsPage {
        #[template_child]
        pub(crate) toolstyle_verticalspace_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) toolstyle_offsetcamera_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) toolstyle_zoom_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnToolsPage {
        const NAME: &'static str = "RnToolsPage";
        type Type = super::RnToolsPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnToolsPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnToolsPage {}
}

glib::wrapper! {
    pub(crate) struct RnToolsPage(ObjectSubclass<imp::RnToolsPage>)
        @extends gtk4::Widget;
}

impl Default for RnToolsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnToolsPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn tool_style(&self) -> Option<ToolStyle> {
        let imp = self.imp();

        if imp.toolstyle_verticalspace_toggle.is_active() {
            Some(ToolStyle::VerticalSpace)
        } else if imp.toolstyle_offsetcamera_toggle.is_active() {
            Some(ToolStyle::OffsetCamera)
        } else if imp.toolstyle_zoom_toggle.is_active() {
            Some(ToolStyle::Zoom)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub(crate) fn set_tool_style(&self, style: ToolStyle) {
        let imp = self.imp();

        match style {
            ToolStyle::VerticalSpace => imp.toolstyle_verticalspace_toggle.set_active(true),
            ToolStyle::OffsetCamera => imp.toolstyle_offsetcamera_toggle.set_active(true),
            ToolStyle::Zoom => imp.toolstyle_zoom_toggle.set_active(true),
        }
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.toolstyle_verticalspace_toggle.connect_toggled(clone!(@weak appwindow => move |toggle| {
            if toggle.is_active() {
                appwindow.active_tab_wrapper().canvas().engine_mut().pens_config.tools_config.style = ToolStyle::VerticalSpace;
            }
        }));

        imp.toolstyle_offsetcamera_toggle.connect_toggled(clone!(@weak appwindow => move |toggle| {
            if toggle.is_active() {
                appwindow.active_tab_wrapper().canvas().engine_mut().pens_config.tools_config.style = ToolStyle::OffsetCamera;
            }
        }));

        imp.toolstyle_zoom_toggle.connect_toggled(clone!(@weak appwindow => move |toggle| {
            if toggle.is_active() {
                appwindow.active_tab_wrapper().canvas().engine_mut().pens_config.tools_config.style = ToolStyle::Zoom;
            }
        }));
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        let tools_config = active_tab
            .canvas()
            .engine_ref()
            .pens_config
            .tools_config
            .clone();

        self.set_tool_style(tools_config.style);
    }
}
