import os

import pytest
from rust_routing import RustRouting


@pytest.fixture(scope="session")
def rust():
    binary = os.environ.get("AVATAR_ROUTING_BINARY")
    if not binary:
        pytest.skip("AVATAR_ROUTING_BINARY names the Rust avatar-routing binary")
    routing = RustRouting(binary)
    yield routing
    routing.close()
