"""Input to the disposable Xvfb display used by the desktop soak, never a user desktop."""
import ctypes as c
class Input:
    def __init__(self):
        self.x=c.CDLL('libX11.so.6'); self.t=c.CDLL('libXtst.so.6')
        self.x.XOpenDisplay.restype=c.c_void_p
        self.d=self.x.XOpenDisplay(None)
        if not self.d: raise RuntimeError('Run soak under xvfb-run')
        for name,args,result in [
            ('XDefaultRootWindow',[c.c_void_p],c.c_ulong),
            ('XQueryTree',[c.c_void_p,c.c_ulong,c.POINTER(c.c_ulong),c.POINTER(c.c_ulong),c.POINTER(c.POINTER(c.c_ulong)),c.POINTER(c.c_uint)],c.c_int),
            ('XFetchName',[c.c_void_p,c.c_ulong,c.POINTER(c.c_char_p)],c.c_int),
            ('XFree',[c.c_void_p],c.c_int),
            ('XSetInputFocus',[c.c_void_p,c.c_ulong,c.c_int,c.c_ulong],c.c_int),
            ('XStringToKeysym',[c.c_char_p],c.c_ulong),
            ('XKeysymToKeycode',[c.c_void_p,c.c_ulong],c.c_uint),
            ('XFlush',[c.c_void_p],c.c_int)]:
            f=getattr(self.x,name); f.argtypes=args; f.restype=result
        self.t.XTestFakeKeyEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
    def find(self):
        def visit(w):
            name=c.c_char_p()
            if self.x.XFetchName(self.d,w,c.byref(name)) and name.value:
                title=name.value; self.x.XFree(name)
                if title==b'Little Forge': return w
            root=c.c_ulong();parent=c.c_ulong();children=c.POINTER(c.c_ulong)();count=c.c_uint()
            if not self.x.XQueryTree(self.d,w,c.byref(root),c.byref(parent),c.byref(children),c.byref(count)): return None
            ids=[children[i] for i in range(count.value)]
            if children: self.x.XFree(children)
            for child in ids:
                match=visit(child)
                if match: return match
        return visit(self.x.XDefaultRootWindow(self.d))
    def keys(self,window,keys):
        self.x.XSetInputFocus(self.d,window,1,0)
        for key in keys:
            code=self.x.XKeysymToKeycode(self.d,self.x.XStringToKeysym(key.encode()))
            self.t.XTestFakeKeyEvent(self.d,code,1,0);self.t.XTestFakeKeyEvent(self.d,code,0,0)
        self.x.XFlush(self.d)
