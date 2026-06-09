# TODO

## P0 已完成

- [x] 路径硬编码 → 支持 `GALLERY_PATH` 环境变量和 `--gallery` 参数
- [x] `intention→situation` 映射：文件名即情境名，去掉 O(n²) UUID 扫描
- [x] 清理 `serde_json` 和 `uuid` 未使用依赖
- [x] 添加 `--help` 参数
- [x] `report <week>` 命令：按六特征模板输出结构化周报

## P1 待完成

- [ ] `relate <week>` 命令：引入 LLM 推理情境间关系
- [ ] 跨周差异分析：`diff W22 W23`

## P2 待完成

- [ ] 心智模型推理：识别跨情境反复出现的模式
