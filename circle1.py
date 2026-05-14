import sympy
(theta,
 radius_scaled,
 x1,
 y1,
 scaling2,
 center_x,
 center_y) = sympy.symbols("""
                            theta,
                            radius_scaled,
                            x1,
                            y1,
                            scaling2,
                            center_x,
                            center_y""")

xi = radius_scaled * sympy.cos(theta)
yi = radius_scaled * sympy.sin(theta)

r2 = xi - x1
alpha = (yi - y1) * scaling2
xf = center_x + r2 * sympy.cos(alpha)
yf = center_y + r2 * sympy.sin(alpha)


dxdtheta = sympy.simplify(sympy.diff(xf, theta))
print("dx/dtheta", dxdtheta)
dydtheta = sympy.simplify(sympy.diff(yf, theta))
print("dy/dtheta", dydtheta)

print("dy/dx", sympy.simplify(dydtheta/dxdtheta))