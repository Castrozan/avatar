use avatar_mac::failure::Failure;
use avatar_mac::meeting::CallDefaults;
use serde_json::json;

use super::World;

impl CallDefaults for World {
    fn route_defaults_to_the_call(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["route_defaults_to_the_call"]));
        self.fails("route_defaults_to_the_call")
    }

    fn restore_defaults(&mut self) -> Result<(), Failure> {
        self.log.push(json!(["restore_defaults"]));
        self.fails("restore_defaults")
    }
}
