import 'package:flutter_test/flutter_test.dart';

import 'package:app_mobile/main.dart';

void main() {
  // 只渲染不点击：点击会调用 FFI，单元测试环境里没有加载原生库。
  testWidgets('renders the shared-core entry', (WidgetTester tester) async {
    await tester.pumpWidget(const AppMobile());

    expect(find.text('Rust AI Template'), findsOneWidget);
    expect(find.text('运行共享核心'), findsOneWidget);
  });
}
