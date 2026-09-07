#import <Cocoa/Cocoa.h>
#include <dlfcn.h>
#include <cstdio>
#include <vector>
#include "pluginterfaces/base/ipluginbase.h"
#include "pluginterfaces/gui/iplugview.h"
#include "pluginterfaces/vst/ivsteditcontroller.h"
using namespace Steinberg;
int main(int argc, char** argv) {
  @autoreleasepool {
    [NSApplication sharedApplication];
    std::vector<IPlugView*> views;
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
        NSView* parent=[[NSView alloc] initWithFrame:NSMakeRect(0,0,640,400)];
        fprintf(stderr,"ATTACH %s\n",info.name);
        result=view->attached((__bridge void*)parent,"NSView");
        fprintf(stderr,"ATTACHED result=%d\n",result);
        if (result!=kResultOk) return 5;
        if (getenv("PROBE_KEY_PASSTHROUGH")) {
          if (view->onKeyDown(' ', 0, 0)!=kResultFalse) return 11;
          if (view->onKeyDown('x', 0, 0)!=kResultFalse) return 12;
          fprintf(stderr, "PASS unused Space and character passthrough\n");
        }
        for (int cycle=0; cycle<3; ++cycle) {
          ViewRect size{0,0,640 + cycle * 32,400 + cycle * 20};
          if (view->checkSizeConstraint(&size)!=kResultOk || view->onSize(&size)!=kResultOk) return 7;
          for (NSView* child in [parent subviews]) { [child display]; }
          if (view->removed()!=kResultOk || [[parent subviews] count]!=0) return 8;
          if (view->attached((__bridge void*)parent,"NSView")!=kResultOk) return 9;
        }
        if (getenv("PROBE_CLOSE_EACH")) {
          view->removed(); view->release(); controller->terminate(); controller->release();
          fprintf(stderr,"CLOSED %s\n",info.name);
        } else { views.push_back(view); controllers.push_back(controller); }
        opened=true; break;
      }
      if (!opened) return 6;
    }
    for (auto it=views.rbegin();it!=views.rend();++it) { (*it)->removed(); (*it)->release(); }
    for (auto c:controllers) { c->terminate(); c->release(); }
    fprintf(stderr,"PASS attach, resize, close/reopen and remove\n");
  }
  return 0;
}
