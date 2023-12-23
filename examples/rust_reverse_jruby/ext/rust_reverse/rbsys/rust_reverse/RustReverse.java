package rbsys.rust_reverse;

import org.jruby.*;
import org.jruby.anno.JRubyMethod;
import org.jruby.anno.JRubyModule;
import org.jruby.runtime.ThreadContext;
import org.jruby.runtime.builtin.IRubyObject;

@SuppressWarnings("serial")
@JRubyModule(name = "RustReverse")
public class RustReverse {

    private static native String reverseNative(String input);

    //    https://github.com/jruby/jruby/wiki/JRubyMethod_Signatures
    //    https://github.com/jruby/jruby/wiki/Method-Signatures-and-Annotations-in-JRuby-extensions

    // It should work like rb_define_module_function
    //    defines a module function, which are private AND singleton methods of the module
    @JRubyMethod(name = "reverse", module = true)
    public static IRubyObject reverse(ThreadContext context, IRubyObject self, RubyString name) {
        return RubyString.newString(context.getRuntime(), RustReverse.reverseNative(name.asJavaString()));
    }
}
