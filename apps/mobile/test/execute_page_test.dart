import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app_mobile/main.dart';

/// 用假执行器把页面挂起来：widget 测试里没有原生库，真调 FFI 会直接崩。
Future<void> pumpPage(WidgetTester tester, Executor executor) {
  return tester.pumpWidget(
    MaterialApp(home: ExecutePage(executor: executor)),
  );
}

void main() {
  testWidgets('shows the message returned by the shared core', (
    WidgetTester tester,
  ) async {
    await pumpPage(tester, (String input) => '已处理 ${input.length} 个字符');

    await tester.enterText(find.byType(TextField), 'hello');
    await tester.tap(find.text('运行共享核心'));
    await tester.pump();

    expect(find.text('已处理 5 个字符'), findsOneWidget);
    expect(find.byIcon(Icons.check_circle_outline), findsOneWidget);
    expect(find.byIcon(Icons.error_outline), findsNothing);
  });

  testWidgets('renders the core error text and switches to the error state', (
    WidgetTester tester,
  ) async {
    await pumpPage(
      tester,
      (String input) => throw AnyhowException('输入不能为空，例如：hello'),
    );

    await tester.enterText(find.byType(TextField), '   ');
    await tester.tap(find.text('运行共享核心'));
    await tester.pump();

    expect(find.text('输入不能为空，例如：hello'), findsOneWidget);
    expect(find.byIcon(Icons.error_outline), findsOneWidget);
    expect(find.byIcon(Icons.check_circle_outline), findsNothing);
  });

  testWidgets('submitting from the keyboard runs the same path', (
    WidgetTester tester,
  ) async {
    final List<String> seen = <String>[];
    await pumpPage(tester, (String input) {
      seen.add(input);
      return 'ok';
    });

    await tester.enterText(find.byType(TextField), 'from-keyboard');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(seen, <String>['from-keyboard']);
    expect(find.text('ok'), findsOneWidget);
  });
}
