import { NavbarSimple } from './ui/navbar';

import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { DevicesPage } from './ui/pages/devices';
import { ControlsPage } from './ui/pages/controls';
import { MeasurementsPage } from './ui/pages/measurements';
import { GraphsPage } from './ui/pages/graphs';
import { TelemetryPage } from './ui/pages/telemetry';

export function App() {
    return (
        // <BrowserRouter>
        //     <NavbarSimple />

        //     <Routes>
        //         <Route path="/" element={<Navigate to="/devices" replace />} />

        //         <Route path="/devices" element={<DevicesPage />} />
        //         <Route path="/controls" element={<ControlsPage />} />
        //         <Route path="/measurements" element={<MeasurementsPage />} />
        //         <Route path="/graphs" element={<GraphsPage />} />
        //         <Route path="/telemetry" element={<TelemetryPage />} />
        //     </Routes>
        // </BrowserRouter>
        <BrowserRouter>
            <div style={{ display: "flex", height: "100vh" }}>
                <NavbarSimple />

                <div style={{ flex: 1, overflow: "auto" }}>
                    <Routes>
                        <Route path="/" element={<Navigate to="/devices" replace />} />
                        <Route path="/devices" element={<DevicesPage />} />
                        <Route path="/controls" element={<ControlsPage />} />
                        <Route path="/measurements" element={<MeasurementsPage />} />
                        <Route path="/graphs" element={<GraphsPage />} />
                        <Route path="/telemetry" element={<TelemetryPage />} />
                    </Routes>
                </div>
            </div>
        </BrowserRouter>
    );
}
