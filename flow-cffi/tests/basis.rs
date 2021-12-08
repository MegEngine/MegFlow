mod utils;

use inline_c::assert_cxx;
use std::env;

#[test]
fn test_version() {
    utils::ready_cpp();
    (assert_cxx! {
        #include "megflow.h"
        #include <iostream>

        int main() {
            std::cout << MGF_version();
            return 0;
        }
    })
    .success()
    .stdout(env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_transform() -> std::io::Result<()> {
    utils::ready_cpp();
    env::set_var("INLINE_C_RS_TESTROOT", env!("CARGO_MANIFEST_DIR"));
    (assert_cxx! {
        #include "megflow.h"
        #include <iostream>
        #include <stdlib.h>
        #include <string>

        void ck(MGFStatus ret) {
            if ((ret) != MGF_STATUS_SUCCESS) {
                char buf[100];
                MGF_error_message(buf, 100);
                std::cout << buf << std::endl;
                abort();
            }
        }

        void* clone_func(void* p) { return p; }
        void release_func(void* p) { (void)p; abort(); }

        int main() {
            MGFGraph graph;
            char buf[100];
            auto root = getenv("TESTROOT");
            auto path = std::string(root) + "/tests/" + "transform.toml";
            ck(MGF_load_graph(path.c_str(), &graph));
            ck(MGF_start_graph(graph));

            MGFMessage message = MGFMessage {
                ptr: buf,
                clone_func: clone_func,
                release_func: release_func,
            };
            ck(MGF_send_message(graph, "inp", message));

            MGFMessage result {};
            ck(MGF_recv_message(graph, "out", &result));

            ck(MGF_close_and_wait_graph(graph));

            return message.ptr != result.ptr;
        }
    })
    .success()
    .code(0);
    Ok(())
}

#[test]
fn test_broadcast() -> std::io::Result<()> {
    utils::ready_cpp();
    env::set_var("INLINE_C_RS_TESTROOT", env!("CARGO_MANIFEST_DIR"));
    (assert_cxx! {
        #include "megflow.h"
        #include <iostream>
        #include <stdlib.h>
        #include <vector>
        #include <string>

        void ck(MGFStatus ret) {
            if ((ret) != MGF_STATUS_SUCCESS) {
                char buf[100];
                MGF_error_message(buf, 100);
                std::cout << buf << std::endl;
                abort();
            }
        }

        void* clone_func(void* p) {
            auto ptr = (std::vector<uint8_t>*)p;
            auto cloned = new std::vector<uint8_t>(*ptr);
            return cloned;
        }
        void release_func(void* p) { (void)p; abort(); }

        int main() {
            MGFGraph graph;
            auto root = getenv("TESTROOT");
            auto path = std::string(root) + "/tests/" + "broadcast.toml";
            ck(MGF_load_graph(path.c_str(), &graph));
            ck(MGF_start_graph(graph));

            MGFMessage message = MGFMessage {
                ptr: new std::vector<uint8_t>(10),
                clone_func: clone_func,
                release_func: release_func,
            };
            ck(MGF_send_message(graph, "inp", message));

            MGFMessage result {};
            ck(MGF_recv_message(graph, "out1", &result));
            delete ((std::vector<uint8_t>*)result.ptr);
            ck(MGF_recv_message(graph, "out2", &result));
            delete ((std::vector<uint8_t>*)result.ptr);

            ck(MGF_close_and_wait_graph(graph));

            return 0;
        }
    })
    .success()
    .code(0);
    Ok(())
}

#[test]
fn test_release() -> std::io::Result<()> {
    utils::ready_cpp();
    env::set_var("INLINE_C_RS_TESTROOT", env!("CARGO_MANIFEST_DIR"));
    (assert_cxx! {
        #include "megflow.h"
        #include <iostream>
        #include <stdlib.h>
        #include <vector>
        #include <atomic>
        #include <string>

        void ck(MGFStatus ret) {
            if ((ret) != MGF_STATUS_SUCCESS) {
                char buf[100];
                MGF_error_message(buf, 100);
                std::cout << buf << std::endl;
                abort();
            }
        }

        void* clone_func(void* p) {
            auto ptr = (std::vector<uint8_t>*)p;
            auto cloned = new std::vector<uint8_t>(*ptr);
            return cloned;
        }

        static std::atomic<int> count {0};

        void release_func(void* p) {
            count+=1;
            delete ((std::vector<uint8_t>*)p);
        }

        int main() {
            MGFGraph graph;
            auto root = getenv("TESTROOT");
            auto path = std::string(root) + "/tests/" + "noop.toml";
            ck(MGF_load_graph(path.c_str(), &graph));
            ck(MGF_start_graph(graph));

            MGFMessage message = MGFMessage {
                ptr: new std::vector<uint8_t>(10),
                clone_func: clone_func,
                release_func: release_func,
            };
            ck(MGF_send_message(graph, "inp", message));

            ck(MGF_close_and_wait_graph(graph));

            return count != 2;
        }
    })
    .success()
    .code(0);
    Ok(())
}
