#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  MGF_PLUGIN_TYPE_PYTHON,
} MGFPluginType;

typedef enum {
  MGF_STATUS_SUCCESS,
  MGF_STATUS_LOAD_FAULT,
  MGF_STATUS_DISCONNECTED,
  MGF_STATUS_NULL_POINTER,
  MGF_STATUS_NO_EXIST_KEY,
  MGF_STATUS_NO_RUNNING_GRAPH,
  MGF_STATUS_INTERNAL,
} MGFStatus;

typedef void *MGFGraph;

typedef struct {
  const char *plugin_path;
  const char *module_path;
  MGFPluginType ty;
} MGFLoaderConfig;

typedef struct {
  void *ptr;
  void *(*clone_func)(void*);
  void (*release_func)(void*);
} MGFMessage;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Get the version of the libflow_cffi.so, e.g. "1.0.0"
 */
const char *MGF_version(void);

/**
 * Load a graph.
 */
MGFStatus MGF_load_graph(const char *config_path, MGFGraph *cgraph);

/**
 * Load a graph with plugins.
 */
MGFStatus MGF_load_graph_with_plugins(const char *config_path,
                                      MGFLoaderConfig plugin_option,
                                      MGFGraph *cgraph);

/**
 * Start the graph.
 * # Safety
 * This function is not thread safe
 */
MGFStatus MGF_start_graph(MGFGraph graph);

/**
 * Close and wait for the graph to run to completion.
 * # Safety
 * 1. This function will free the graph at the end of the function.
 * 2. This function is not thread safe
 */
MGFStatus MGF_close_and_wait_graph(MGFGraph graph);

/**
 * Send a message to a specific port of the graph
 */
MGFStatus MGF_send_message(MGFGraph graph, const char *name, MGFMessage message);

/**
 * Receiver a message from a specific port of the graph
 */
MGFStatus MGF_recv_message(MGFGraph graph, const char *name, MGFMessage *cmessage);

/**
 * Clear the last error.
 */
void MGF_clear_last_error(void);

/**
 * Get the length of the last error message in bytes when encoded as UTF-8,
 * including the trailing null.
 */
int MGF_last_error_length(void);

/**
 * Peek at the most recent error and write its error message
 * into the provided buffer as a UTF-8 encoded string.
 *
 * This returns the number of bytes written, or `-1` if there was an error.
 */
int MGF_error_message(char *buf, int length);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
