import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';

import 'src/rust/api/execute.dart';
import 'src/rust/frb_generated.dart';

Future<void> main() async {
  // RustLib.init() 会调用 Rust 侧的 #[frb(init)] 函数，必须在任何绑定调用之前完成。
  await RustLib.init();
  runApp(const AppMobile());
}

/// 页面调用共享核心的方式：输入一段文本，返回一行结果或抛出 [AnyhowException]。
///
/// 抽成函数类型是为了可测：widget 测试环境里没有原生库，直接调 FFI 会炸；
/// 注入一个假的执行器，就能把「成功显示 message、失败显示核心文案」这条界面逻辑测住。
typedef Executor = String Function(String input);

/// 生产路径：调用 Rust 侧的同步导出函数。
String runSharedCore(String input) => execute(input: input);

class AppMobile extends StatelessWidget {
  const AppMobile({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Rust AI Template',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF2563EB),
          brightness: Brightness.dark,
        ),
        scaffoldBackgroundColor: const Color(0xFF18181B),
      ),
      home: const ExecutePage(),
    );
  }
}

class ExecutePage extends StatefulWidget {
  const ExecutePage({super.key, this.executor = runSharedCore});

  /// 默认走共享核心；测试注入假实现即可覆盖成功与失败两条路径。
  final Executor executor;

  @override
  State<ExecutePage> createState() => _ExecutePageState();
}

class _ExecutePageState extends State<ExecutePage> {
  final TextEditingController _controller = TextEditingController(text: 'hello');
  String? _result;
  bool _isError = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _run() {
    // execute 是同步函数：校验与文案都在 app-core，Dart 侧只负责展示。
    setState(() {
      try {
        _result = widget.executor(_controller.text);
        _isError = false;
      } on AnyhowException catch (error) {
        _result = error.message;
        _isError = true;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Rust AI Template')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Text(
                '移动端：Flutter + flutter_rust_bridge，业务逻辑在 app-core',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: const Color(0xFFA1A1AA),
                ),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: _controller,
                decoration: const InputDecoration(
                  labelText: '共享核心输入',
                  helperText: '空白会被拒绝，上限 4096 字符',
                  border: OutlineInputBorder(),
                ),
                onSubmitted: (_) => _run(),
              ),
              const SizedBox(height: 12),
              FilledButton(
                onPressed: _run,
                child: const Text('运行共享核心'),
              ),
              if (_result != null) ...<Widget>[
                const SizedBox(height: 20),
                _ResultCard(message: _result!, isError: _isError),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

class _ResultCard extends StatelessWidget {
  const _ResultCard({required this.message, required this.isError});

  final String message;
  final bool isError;

  @override
  Widget build(BuildContext context) {
    final Color accent =
        isError ? const Color(0xFFF87171) : const Color(0xFF4ADE80);

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: const Color(0xFF27272A),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Icon(
            isError ? Icons.error_outline : Icons.check_circle_outline,
            color: accent,
            size: 20,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(message, style: TextStyle(color: accent)),
          ),
        ],
      ),
    );
  }
}
