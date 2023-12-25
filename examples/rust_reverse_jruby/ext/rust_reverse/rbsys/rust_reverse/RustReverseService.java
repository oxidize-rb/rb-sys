package rbsys.rust_reverse;

import org.jruby.*;
import org.jruby.runtime.load.BasicLibraryService;

public class RustReverseService implements BasicLibraryService {
    public static void systemLoad(String libPath) {
        System.load(libPath);
    }

    @Override
    public boolean basicLoad(final Ruby ruby) {
        RubyModule klass = ruby.defineModule("RustReverse");
        klass.defineAnnotatedMethods(RustReverse.class);
        return true;
    }
}
