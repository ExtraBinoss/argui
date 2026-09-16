#import "ArguiWidgetGallery.h"

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;
    @autoreleasepool {
        argui_ios_activity_bridge_anchor();
        start_argui_widget_gallery();
    }
    return 0;
}
