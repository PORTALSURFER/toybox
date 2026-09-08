#import <Cocoa/Cocoa.h>
#include <dlfcn.h>
#include <cstdio>
#include <cstdlib>
#include <cmath>
#include <vector>
#include "pluginterfaces/base/ipluginbase.h"
#include "pluginterfaces/gui/iplugview.h"
#include "pluginterfaces/vst/ivsteditcontroller.h"
using namespace Steinberg;
int main(int argc, char** argv) {
  if (argc<2) { fprintf(stderr,"usage: plugin-editor-probe <VST3 binary> [additional binary...]\n"); return 1; }
  @autoreleasepool {
    [NSApplication sharedApplication];
    std::vector<NSWindow*> windows;
    std::vector<IPlugView*> views;
    auto pump = [](double seconds) {
      NSDate* deadline = [NSDate dateWithTimeIntervalSinceNow:seconds];
      do {
        NSEvent* event = [NSApp nextEventMatchingMask:NSEventMaskAny
          untilDate:[NSDate dateWithTimeIntervalSinceNow:0.005]
          inMode:NSDefaultRunLoopMode dequeue:YES];
        if (event) [NSApp sendEvent:event];
        [NSApp updateWindows];
      } while ([deadline timeIntervalSinceNow] > 0);
    };
    std::vector<Vst::IEditController*> controllers;
    for (int n=1; n<argc; ++n) {
      fprintf(stderr, "LOAD %s\n", argv[n]);
      void* lib=dlopen(argv[n], RTLD_NOW|RTLD_LOCAL);
      if (!lib) { fprintf(stderr,"%s\n",dlerror()); return 2; }
      auto entry=(bool(*)(CFBundleRef))dlsym(lib,"bundleEntry");
      if (entry) {
        NSString* binary=[NSString stringWithUTF8String:argv[n]];
        NSString* bundle=[[[binary stringByDeletingLastPathComponent] stringByDeletingLastPathComponent] stringByDeletingLastPathComponent];
        CFBundleRef ref=CFBundleCreate(kCFAllocatorDefault, (__bridge CFURLRef)[NSURL fileURLWithPath:bundle]);
        if (!ref || !entry(ref)) return 10;
        // Keep the module bundle alive until process exit, like a loaded host module.
      }
      auto get=(IPluginFactory*(*)())dlsym(lib,"GetPluginFactory");
      if (!get) return 3;
      auto factory=get();
      TUID iid=INLINE_UID(0xDCD7BBE3,0x7742448D,0xA874AACC,0x979C759E);
      bool opened=false;
      for (int i=0;i<factory->countClasses();++i) {
        PClassInfo info{}; factory->getClassInfo(i,&info);
        Vst::IEditController* controller=nullptr;
        auto result=factory->createInstance(info.cid,iid,(void**)&controller);
        if (result!=kResultOk || !controller) continue;
        fprintf(stderr,"CONTROLLER %s initialize=%d\n",info.name,controller->initialize(nullptr));
        auto view=controller->createView("editor");
        if (!view) return 4;
        ViewRect preferred{};
        if (view->getSize(&preferred)!=kResultOk) return 13;
        NSRect frame=NSMakeRect(0,0,preferred.right-preferred.left,preferred.bottom-preferred.top);
        NSWindow* window=[[NSWindow alloc] initWithContentRect:frame
          styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskResizable
          backing:NSBackingStoreBuffered defer:NO];
        [window setReleasedWhenClosed:NO];
        [window setTitle:[NSString stringWithFormat:@"GainSnap GPUI probe %d",n]];
        NSView* parent=[window contentView];
        windows.push_back(window);
        if (getenv("PROBE_VISIBLE")) {
          [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
          [window makeKeyAndOrderFront:nil];
          [NSApp activateIgnoringOtherApps:YES];
        }
        fprintf(stderr,"ATTACH %s\n",info.name);
        result=view->attached((__bridge void*)parent,"NSView");
        fprintf(stderr,"ATTACHED result=%d\n",result);
        if (result!=kResultOk) return 5;
        pump(0.05);
        if (getenv("PROBE_KEY_PASSTHROUGH")) {
          if (view->onKeyDown(' ', 0, 0)!=kResultFalse) return 11;
          if (view->onKeyDown('x', 0, 0)!=kResultFalse) return 12;
          fprintf(stderr, "PASS unused Space and character passthrough\n");
        }
        for (int cycle=0; cycle<3; ++cycle) {
          ViewRect size{0,0,640 + cycle * 32,400 + cycle * 20};
          if (view->checkSizeConstraint(&size)!=kResultOk || view->onSize(&size)!=kResultOk) return 7;
          [window setContentSize:NSMakeSize(size.right-size.left,size.bottom-size.top)];
          pump(0.02);
          for (NSView* child in [parent subviews]) { [child display]; }
          if (view->removed()!=kResultOk || [[parent subviews] count]!=0) return 8;
          if (view->attached((__bridge void*)parent,"NSView")!=kResultOk) return 9;
        }
        if (view->onSize(&preferred)!=kResultOk) return 14;
        [window setContentSize:NSMakeSize(preferred.right-preferred.left,preferred.bottom-preferred.top)];
        pump(0.02);
        if (getenv("PROBE_GAIN_SNAP_INPUT")) {
          [window makeKeyAndOrderFront:nil];
          const NSInteger number=[window windowNumber];
          auto key=[&](NSString* text,unsigned short code,NSEventModifierFlags modifiers,bool standalone_callback=false) {
            NSEvent* down=[NSEvent keyEventWithType:NSEventTypeKeyDown location:NSZeroPoint
              modifierFlags:modifiers timestamp:NSProcessInfo.processInfo.systemUptime windowNumber:number context:nil
              characters:text charactersIgnoringModifiers:text isARepeat:NO keyCode:code];
            [window sendEvent:down];
            // This direct ABI call has no shared native OS event token.
            if (standalone_callback) view->onKeyDown(0,12,0);
            NSEvent* up=[NSEvent keyEventWithType:NSEventTypeKeyUp location:NSZeroPoint
              modifierFlags:modifiers timestamp:NSProcessInfo.processInfo.systemUptime windowNumber:number context:nil
              characters:text charactersIgnoringModifiers:text isARepeat:NO keyCode:code];
            [window sendEvent:up];
            pump(0.02);
          };
          for (NSEventType type : {NSEventTypeLeftMouseDown,NSEventTypeLeftMouseUp}) {
            NSEvent* click=[NSEvent mouseEventWithType:type location:NSMakePoint(48,28)
              modifierFlags:0 timestamp:0 windowNumber:number context:nil
              eventNumber:1 clickCount:1 pressure:1];
            [window sendEvent:click];
          }
          pump(0.02);
          key(@"a",0,NSEventModifierFlagCommand);
          key(@"-",27,0); key(@"1",18,0); key(@"5",23,0); key(@"\r",36,0);
          const double typed=-36.0+36.0*controller->getParamNormalized(1);
          if (std::fabs(typed-(-15.0))>0.001) {
            fprintf(stderr,"FAIL native target entry expected -15 got %.6f\n",typed); return 16;
          }
          key(@"\uF700",126,0);
          key(@"\uF701",125,NSEventModifierFlagShift);
          const double stepped=-36.0+36.0*controller->getParamNormalized(1);
          if (std::fabs(stepped-(-14.1))>0.001) {
            fprintf(stderr,"FAIL native arrows expected -14.1 got %.6f\n",stepped); return 17;
          }
          fprintf(stderr,"PASS native GainSnap selection, target typing, commit, arrows and Shift-arrows\n");
          view->onKeyDown('a',0,4); view->onKeyUp('a',0,4);
          for (char ch : {'-','1','8'}) { view->onKeyDown(ch,0,0); view->onKeyUp(ch,0,0); }
          view->onKeyDown(0,4,0); view->onKeyUp(0,4,0);
          pump(0.02);
          const double callback_target=-36.0+36.0*controller->getParamNormalized(1);
          if (std::fabs(callback_target-(-18.0))>0.001) {
            fprintf(stderr,"FAIL VST3 callback target entry expected -18 got %.6f\n",callback_target); return 18;
          }
          key(@"\uF700",126,0,true);
          const double independent_target=-36.0+36.0*controller->getParamNormalized(1);
          if (std::fabs(independent_target-(-16.0))>0.001) {
            fprintf(stderr,"FAIL independent native/VST3 Up expected -16 got %.6f\n",independent_target); return 19;
          }
          fprintf(stderr,"PASS VST3 callback text entry and standalone callback fallback\n");
        }
        if (getenv("PROBE_CLOSE_EACH")) {
          view->removed(); view->release(); controller->terminate(); controller->release();
          fprintf(stderr,"CLOSED %s\n",info.name);
        } else { views.push_back(view); controllers.push_back(controller); }
        opened=true; break;
      }
      if (!opened) return 6;
    }
    if (const char* seconds=getenv("PROBE_INTERACTIVE_SECONDS")) {
      double duration=std::strtod(seconds,nullptr);
      if (duration>0 && duration<=600) pump(duration);
    }
    if (getenv("PROBE_GAIN_SNAP_PARAMS") || getenv("PROBE_EXPECT_TARGET_DB")) {
    for (auto controller:controllers) {
      const double target=controller->getParamNormalized(1);
      fprintf(stderr,"PARAM target normalized=%.9f plain=%.3f match=%.0f rms=%.0f\n",
        target,-36.0+36.0*target,controller->getParamNormalized(2),controller->getParamNormalized(4));
      if (const char* expected=getenv("PROBE_EXPECT_TARGET_DB")) {
        if (std::fabs((-36.0+36.0*target)-std::strtod(expected,nullptr))>0.001) return 15;
      }
    }
    }
    for (auto it=views.rbegin();it!=views.rend();++it) { (*it)->removed(); (*it)->release(); }
    for (auto c:controllers) { c->terminate(); c->release(); }
    for (auto window:windows) { [window close]; [window release]; }
    fprintf(stderr,"PASS attach, resize, close/reopen and remove\n");
  }
  return 0;
}
