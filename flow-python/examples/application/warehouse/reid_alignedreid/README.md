# How to test AlignedReid Model

Download models and soft link
```bash
$ ln -s ${DOWNLOAD_DIR} flow-python/examples/models
```

Run

```bash
$ cd flow-python/examples/warehouse
$ python3 -m reid_alignedreid.main ../models/aligned_reid.pkl  reid_alignedreid/image/positive1.jpg  reid_alignedreid/image/positive2.jpg  reid_alignedreid/image/negative.jpg

10 19:07:09 WRN Unused params in `strict=False` mode, unused={'local_bn.bias', 'fc.weight', 'local_bn.running_mean', 'local_bn.running_var', 'local_conv.weight', 'local_conv.bias', 'fc.bias', 'local_bn.weight'}
distance_positive: 0.0987139493227005
distance_negtive: 0.2990190088748932
```
