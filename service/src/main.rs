extern crate dbus;
extern crate calc;

use dbus::{Connection, BusType, NameFlag};
use dbus::tree::Factory;
use std::sync::Arc;

// Wrapper function, so that we can use ?
fn run_dbus_service() -> Result<(), dbus::Error> {
    // Acquire session bus instance
    let connection = Connection::get_private(BusType::Session)?;

    // Register a service name
    connection.register_name(
        "cz.sehny.service",
        NameFlag::ReplaceExisting as u32)?;

    // Factory is an abstract construction used for creation of DBus components like methods,
    // objects and interfaces
    let factory = Factory::new_fnmut::<()>();

    let signal = Arc::new(
        factory
            .signal("CalcExecuted", ())
            .sarg::<&str,_>("expression")
    );
    let signal_clone = signal.clone();
    
    // New method "Eval"uate expression
    let method = factory.method("Eval", (),
        // I use move to transfer ownership of the signal variable
        move |m|{
            let n: &str = m.msg.read1()?;
            let res = match calc::eval(n) {
                Ok(val) => format!("{}={}", n, val),
                Err(_) => format!("There was an error in the input expression!"),
            };
            let s = format!("[Calculator service] {}", res);
            let sig = signal
                .msg(m.path.get_name(), m.iface.get_name())
                .append(format!("{}", res));
            Ok(vec!(m.msg.method_return().append1(s), sig))
    })  .inarg::<&str,_>("args")
        .outarg::<&str,_>("result");

    let interface = factory.interface("cz.sehny.calc", ())
        .add_m(method)
        .add_s(signal_clone);

    let object = factory.object_path("/calc", ())
        .introspectable()
        .add(interface);

    // In order to create an object hierarchy, I create a tree, which will than
    // contain objects specified by their paths
    let tree = factory.tree(())
        .add(object);
    tree.set_registered(&connection, true)?;
    connection.add_handler(tree);
    loop {
        connection.incoming(1000).next();
    }
}

fn main() {
    if let Err(e) = run_dbus_service() {
        eprintln!("There was an error in DBus service: {}", e);
    }
}
