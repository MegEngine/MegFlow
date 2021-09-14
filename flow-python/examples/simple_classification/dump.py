import model as resnet_model
import megengine as mge
from megengine import jit
import numpy as np

def dump_static_graph(model, graph_name="model.mge"):
    model.eval()

    data = mge.Tensor(np.random.random((1, 3, 224, 224)))

    @jit.trace(capture_as_const=True)
    def pred_func(data):
        outputs = model(data)
        return outputs

    pred_func(data)
    pred_func.dump(
        graph_name,
        arg_names=["data"],
        optimize_for_inference=True,
        enable_fuse_conv_bias_nonlinearity=True,
    )

def main():
    model = resnet_model.__dict__['resnet18'](pretrained=True)
    dump_static_graph(model)

if __name__ == "__main__":
    main()